import fs from "fs";
import express from "express";
import expressWs from "express-ws";
import path from "path";
import process from "process";
import luaparse from "luaparse";
import msgpack from "@msgpack/msgpack";
import hash from "hash-it";
import bundler from "luabundle";
import * as lua from "luamin";
import pako from "pako";
const luamin = lua.default;
const BuiltinModules = [
    "cc.audio.dfpwm",
    "cc.completion",
    "cc.expect",
    "cc.image.nft",
    "cc.pretty",
    "cc.require",
    "cc.shell.completion",
    "cc.strings"
];
export class SyncServer {
    port;
    project;
    server;
    channelsChanged = [];
    channelHashes = new Map();
    subscribed = new Map();
    files = new Map();
    fileBuffered = new Map();
    latestMessage = new Map();
    requestCount = new Map();
    luaRoot;
    minify;
    maxRequestSize;
    constructor(port, projectPath, luaFilesDir, maxRequestSize = 50000, minify = false) {
        this.port = port;
        this.minify = minify;
        this.luaRoot = luaFilesDir;
        this.project = JSON.parse(fs.readFileSync(path.join(process.cwd(), projectPath), 'utf8'));
        const expr = express();
        this.server = expressWs(expr).app;
        this.maxRequestSize = maxRequestSize;
    }
    setup() {
        this.server.get("/", (req, res) => {
            const channels = [];
            for (const channel of this.project.project)
                channels.push({ channel: channel.channelName, type: channel.type });
            res.send(Buffer.from(msgpack.encode(channels)).toString("base64"));
        });
        this.server.ws("/subscribe", (ws) => {
            ws.once("message", (data, binary) => {
                const decode = Buffer.from(data, "base64");
                const uint = new Uint8Array(decode);
                const decoded = msgpack.decode(uint);
                this.latestMessage.set(ws, data);
                decoded.channels = decoded.channels.filter((v) => this.project.project.find((b) => b.channelName === v));
                if (decoded.channels.length === 0) {
                    ws.close();
                    this.latestMessage.delete(ws);
                    return;
                }
                this.requestCount.set(ws, -1);
                this.subscribed.set(ws, decoded.channels);
                this.newSubscription(ws);
                ws.once("close", () => this.subscribed.delete(ws));
                ws.on("message", (data) => {
                    this.latestMessage.set(ws, data);
                });
            });
        });
        setInterval(() => {
            this.UpdateHashes();
        }, 500);
        this.server.get("/sync.lua", (req, res) => {
            const file = path.join(this.luaRoot, "sync.lua");
            res.send(luamin.minify(bundler.bundle(file, {
                resolveModule: (modu) => {
                    if (modu === "msgpack")
                        return path.join(this.luaRoot, "msgpack.lua");
                    if (modu === "base64")
                        return path.join(this.luaRoot, "base64.lua");
                    if (modu === "libdeflate")
                        return path.join(this.luaRoot, "libdeflate.lua");
                },
                ignoredModuleNames: BuiltinModules
            })));
            /*res.send(bundler.bundle(file, {
              resolveModule: (modu) => {
                if (modu === "msgpack")
                  return path.join(this.luaRoot, "msgpack.lua");
                if (modu === "base64")
                  return path.join(this.luaRoot, "base64.lua");
                if (modu === "libdeflate")
                  return path.join(this.luaRoot, "libdeflate.lua");
              },
              ignoredModuleNames: BuiltinModules
            }))*/
        });
        this.server.listen(this.port, () => {
            console.log(`hosting sync server on port ${this.port}`);
        });
    }
    waitForVariableToBe(value, variableGetter, checkInterval = 100) {
        return new Promise((resolve) => {
            const interval = setInterval(() => {
                if (variableGetter() === value) {
                    clearInterval(interval);
                    resolve();
                }
            }, checkInterval);
        });
    }
    async newSubscription(ws) {
        const subscribedChannels = this.subscribed.get(ws);
        for (const channel of subscribedChannels) {
            const requests = await this.getRequestsForChannel(channel);
            for (const request of requests) {
                const requestCount = this.requestCount.get(ws);
                await this.waitForVariableToBe(`waiting${requestCount + 1}`, () => this.latestMessage.get(ws), 1);
                this.requestCount.set(ws, requestCount + 1);
                const data = pako.deflateRaw(msgpack.encode(request), { level: 9 });
                ws.send(data);
            }
        }
    }
    getChannels() {
        const channels = [];
        for (const item of this.project.project)
            channels.push(item.channelName);
        return channels;
    }
    assembleRequire(statement) {
        const requires = [];
        for (const expr of statement.init) {
            if (expr.type !== "CallExpression")
                continue;
            if (expr.base.type !== "Identifier")
                continue;
            if (expr.base.name !== "require")
                continue;
            let string = "require(";
            const module = expr.arguments[0];
            if (module.type !== "StringLiteral")
                continue;
            string += module.raw;
            string += ")";
            requires.push({
                fullString: string,
                requiredModule: module.raw.replaceAll('"', "")
            });
        }
        return requires;
    }
    preprocess(content) {
        let newContent = content;
        const ast = luaparse.parse(newContent, { luaVersion: "5.2" });
        const requires = ast.body.filter((v) => v.type === "LocalStatement" && v.init.filter((v) => v.type === "CallExpression").find((v) => v.base.type === "Identifier" && v.base.name === "require"));
        const assembled = [];
        for (const req of requires)
            assembled.push(...this.assembleRequire(req));
        const modifiedRequires = [];
        for (const req of assembled) {
            if (BuiltinModules.includes(req.requiredModule.replaceAll('"', "")))
                continue;
            let replacedRequire = req.requiredModule.replaceAll('"', "");
            if (!replacedRequire.startsWith("/"))
                replacedRequire = `/${replacedRequire}`;
            const modifiedString = `require("${replacedRequire}")`;
            modifiedRequires.push({
                original: req.fullString,
                replacement: modifiedString
            });
        }
        for (const modified of modifiedRequires) {
            newContent = newContent.replace(modified.original, modified.replacement);
        }
        if (this.minify)
            newContent = luamin.minify(newContent);
        return newContent;
    }
    /*private splitStringIntoChunks(str: string, chunkSize: number) {
      const encoder = new TextEncoder();
      const encoded = encoder.encode(str);
      const chunks: string[] = [];
  
      for (let i = 0; i < encoded.length; i += chunkSize) {
        const chunk = encoded.slice(i, i + chunkSize);
        const decodedChunk = new TextDecoder().decode(chunk);
        chunks.push(decodedChunk);
      }
  
      return chunks;
    }*/
    processFiles(channel) {
        const data = [];
        for (const file of channel.files ?? []) {
            const fdata = fs.readFileSync(path.join(process.cwd(), this.project.rootDir, file.path), 'utf8');
            const processed = this.preprocess(fdata);
            data.push({
                type: channel.type,
                fileData: channel.minify && !this.minify ? luamin.minify(processed) : processed,
                filePath: file.name
            });
            /*data.push({
              type: channel.type,
              fileData: processed,
              filePath: file
            })*/
        }
        const realFiles = [];
        for (const directory of channel.directories ?? []) {
            const dirpath = path.join(process.cwd(), this.project.rootDir, directory);
            const files = fs.readdirSync(dirpath, { recursive: true, encoding: "utf8" });
            for (const file of files) {
                const filepath = path.join(dirpath, file);
                const stat = fs.statSync(filepath);
                if (!stat.isFile())
                    continue;
                realFiles.push(path.join(directory, file));
                const fdata = fs.readFileSync(filepath, 'utf8');
                const processed = this.preprocess(fdata);
                data.push({
                    type: channel.type,
                    fileData: channel.minify && !this.minify ? luamin.minify(processed) : processed,
                    filePath: path.join(directory, file)
                });
                /*data.push({
                  type: channel.type,
                  fileData: processed,
                  filePath: path.join(directory, file)
                })*/
            }
        }
        this.fileBuffered.set(channel, realFiles);
        const removed = (this.files.get(channel) ?? []).filter((v) => !realFiles.includes(v));
        if (removed.length > 0)
            data.push({
                type: "deletion",
                files: removed
            });
        return data;
    }
    updateFiles() {
        this.fileBuffered.forEach((v, k) => this.files.set(k, v));
        this.fileBuffered.clear();
    }
    processLibrary(channel) {
        const pchannel = this.project.project.find((v) => v.channelName === channel);
        if (!pchannel)
            throw `Channel ${channel} does not exist!`;
        if (pchannel.type === "script")
            throw `Script channel ${channel} should not be getting processed in processLibrary!`;
        const channelRequests = [];
        if (pchannel.requiredChannels)
            for (const required of pchannel.requiredChannels) {
                const processed = this.processLibrary(required);
                channelRequests.push(...processed);
            }
        if ("files" in pchannel || "directories" in pchannel) {
            const files = this.processFiles(pchannel);
            channelRequests.push(...files);
        }
        return channelRequests;
    }
    processChannel(channel) {
        const pchannel = this.project.project.find((v) => v.channelName === channel);
        if (!pchannel)
            throw `Channel ${channel} does not exist!`;
        if (pchannel.type === "library")
            throw `Library channel ${channel} should not be getting processed in processChannel!`;
        const channelRequests = [];
        if (pchannel.requiredChannels)
            for (const required of pchannel.requiredChannels) {
                const processed = this.runProcess(required);
                channelRequests.push(...processed);
            }
        const files = this.processFiles(pchannel);
        channelRequests.push(...files);
        return channelRequests;
    }
    runProcess(channel) {
        const ch = this.project.project.find((v) => v.channelName === channel);
        if (!ch)
            throw `Channel ${channel} does not exist!`;
        if (ch.type === "library")
            return this.processLibrary(channel);
        return this.processChannel(channel);
    }
    async chunkRequests(requests) {
        const s1 = Date.now();
        const alreadySeen = [];
        const dedup = requests.filter((v) => {
            if ("filePath" in v) {
                const path = v.filePath;
                const h = hash(v.fileData).toString();
                if (!alreadySeen.includes(`${path}${h}`)) {
                    alreadySeen.push(`${path}${h}`);
                    return true;
                }
            }
            if ("files" in v) {
                const h = hash(v.files).toString();
                if (!alreadySeen.includes(h)) {
                    alreadySeen.push(h);
                    return true;
                }
            }
            return false;
        });
        const creationRequests = dedup.filter((v) => v.type === "library" || v.type === "script").filter((v) => pako.deflateRaw(v.fileData, { level: 9 }).length < this.maxRequestSize).sort((a, b) => b.fileData.length - a.fileData.length);
        console.log(`took ${Date.now() - s1}ms to filter requests`);
        const deletionRequests = dedup.filter((v) => v.type === "deletion");
        const s2 = Date.now();
        const tooLarge = creationRequests.filter((v) => pako.deflateRaw(v.fileData, { level: 9 }).length >= this.maxRequestSize);
        console.log(`took ${Date.now() - s2}ms to filter too large requests`);
        const chunks = [];
        let currentChunkSize = 0;
        let currentChunk = [];
        const s3 = Date.now();
        for (const req of creationRequests) {
            const dataSize = pako.deflateRaw(req.fileData, { level: 9 }).length;
            if (currentChunkSize + dataSize >= this.maxRequestSize) {
                chunks.push(currentChunk);
                currentChunk = [];
                currentChunkSize = 0;
            }
            currentChunkSize += dataSize;
            currentChunk.push(req);
        }
        if (currentChunk.length > 0) {
            chunks.push(currentChunk);
            currentChunk = [];
            currentChunkSize = 0;
        }
        console.log(`took ${Date.now() - s3}ms to chunk creation requests`);
        let currentFileChunkSize = 0;
        let currentFiles = [];
        const files = [];
        for (const req of deletionRequests) {
            const dataSizes = req.files.map((v) => ({ path: v, length: v.length }));
            let currentFiles = [];
            for (const size of dataSizes) {
                const dsize = size.length;
                if (currentFileChunkSize + dsize >= this.maxRequestSize) {
                    files.push(currentFiles);
                    currentFiles = [];
                    currentFileChunkSize = 0;
                }
                currentFileChunkSize += dsize;
                currentFiles.push(size.path);
            }
        }
        if (currentFiles.length > 0)
            files.push(currentFiles);
        for (const deleteChunk of files) {
            chunks.push([
                {
                    type: "deletion",
                    files: deleteChunk
                }
            ]);
        }
        const promises = [];
        const s4 = Date.now();
        for (const req of tooLarge) {
            promises.push(new Promise((resolve) => {
                let resultString = "";
                const dataSplit = req.fileData.split("");
                for (const char of dataSplit) {
                    const resLength = pako.deflateRaw(resultString + char, { level: 9 });
                    if (resLength.length >= this.maxRequestSize) {
                        chunks.push([{
                                type: "chunk",
                                fileData: resultString,
                                filePath: req.filePath
                            }]);
                        resultString = "";
                    }
                    resultString = resultString + char;
                }
                if (resultString.length > 0) {
                    chunks.push([{
                            type: "chunk",
                            fileData: resultString,
                            filePath: req.filePath
                        }]);
                }
                resolve();
            }));
        }
        await Promise.all(promises);
        console.log(`took ${Date.now() - s4}ms to chunk requests that are too large`);
        return chunks.filter((v) => v.length > 0);
    }
    getRequestsForChannel(channel) {
        const pchannel = this.project.project.find((v) => v.channelName === channel);
        if (!pchannel)
            throw `Channel ${channel} does not exist!`;
        const requests = this.runProcess(channel);
        const chunks = this.chunkRequests(requests);
        //return requests;
        return chunks;
    }
    getRequestsForChannelRaw(channel) {
        const pchannel = this.project.project.find((v) => v.channelName === channel);
        if (!pchannel)
            throw `Channel ${channel} does not exist!`;
        const requests = this.runProcess(channel);
        return requests;
    }
    async UpdateHashes() {
        let changed = false;
        for (const channel of this.getChannels()) {
            const requests = this.getRequestsForChannelRaw(channel);
            const channelHash = hash(requests);
            const previousHash = this.channelHashes.get(channel);
            if (channelHash !== previousHash) {
                changed = true;
                this.channelHashes.set(channel, channelHash);
                this.channelsChanged.push(channel);
            }
        }
        if (changed)
            await this.sendForChanged();
    }
    async sendForChanged() {
        const data = new Map();
        for (const channel of this.channelsChanged) {
            const requests = await this.getRequestsForChannel(channel);
            data.set(channel, requests);
        }
        const promises = [];
        this.subscribed.forEach((channels, ws) => {
            promises.push(new Promise(async (resolve) => {
                for (const channel of channels) {
                    if (!data.has(channel))
                        continue;
                    const requests = data.get(channel);
                    for (const request of requests) {
                        const requestCount = this.requestCount.get(ws);
                        await this.waitForVariableToBe(`waiting${requestCount + 1}`, () => this.latestMessage.get(ws), 1);
                        this.requestCount.set(ws, requestCount + 1);
                        ws.send(pako.deflateRaw(msgpack.encode(request), { level: 9 }));
                    }
                }
                resolve();
            }));
        });
        await Promise.all(promises);
        this.channelsChanged = [];
        this.updateFiles();
    }
}
