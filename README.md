# CC:T Syncing Server

A primarily [TypescriptToLua](https://typescripttolua.github.io/) oriented CC:T sync server.

Made because I didn't want to manually enter everything and the other tools I could find didn't serve the sole purpose of syncing files, instead being either a turtle remote access tool or a turtle emulator.

Heavily biased towards my own usecases, and will likely be missing more general-use QOL features.

project.json structure:
```json5
{
    "rootDir": "dir", // The directory to search for files in. This is relative to where you're running the server, so "." would resolve to the current directory, and "build" would resolve to "currentDirectory/build"
    "project": [     // This is where we declare the various channels available to clients.
        {
           "type": "library", // Can be "library" or "script", "library" channels can be requirements for other channels, library or script.
           "files": [
                {
                    "path": "startups/testlib.lua", // Path to file
                    "name": "startup.lua" // Resulting path on CC filesystem
                }
            ], // All the files this channel contains. Optional as long as directories is declared.
           "channelName": "testlib", // The channel the client has to connect to in order to sync these files.
           "directories": ["lib1/subdir"], // Folders that should be watched for this channel. Optional as long as files is declared.
           "requiredChannels": ["testreq"], // The channels required for this channel to function. Circular dependencies are not handled and should be avoided.
           "minify": true // Optional, if true this channel's contents will be minified (does not apply to required channels)
        }
    ]
}
```
Note: Both `files` and `directories` are optional if the channel is a `library` and has `requiredChannels` set. This is for libraries that represent several other libraries joined together.

If you wish to sync multiple channels at once, a single client can request to connect to several channels.

Upon connection, the first packet sent is a "subscribe" packet, detailing the channels the client wishes to sync.

If a "GET" request is sent to "/", we respond with all available channels and their types.

I don't know how to make a sort of "sourcemap" to ensure the client can remove files that are no longer in the project, especially seeing as there's multiple channels to connect to, so auto-removing files is not a feature. If I work on this further, I might try to add that.

This project also sends the entire channel's data every time a file is changed, so it's currently fairly inefficient when it comes to network usage.

## Config 

You can configure the server by editing config.json (in project root, same dir as project.json).

```json5
{
    "port": 10234,
    "minify": false,
    "ngrok": false,
    "maxRequestSize": 50000
}
```

### port
The port to start the Express server on. Used when downloading sync.lua and when connecting to the server for sync.

### minify
If true, minifies all code sent to the client. Useful if your code is larger and needs to be minified to run properly.

Unnecessary if you only need to minify a specific channel, as you can set `"minify"` on a channel to specify whether that channel should be minified.

### ngrok
If true, starts up an ngrok tunnel for the specified port and logs the domain to terminal.

### maxRequestSize
How large (in bytes, I think) a request can be. Defaults to 50kb, can be set larger or smaller depending on the server config.

This attempts to take into account deflated size, but won't be entirely accurate as deflating the entire request chunk will end up smaller than the individual requests themselves.

## Initial client setup

Setting up a client is fairly simple.

In order to get the syncing script, run `wget {host-url}/sync.lua`

`{host-url}` should either be a URL logged in the console (adding http://) or your own domain, depending on how you choose to run this.

If running this on the same computer, you can just run `wget http://localhost:10234/sync.lua` if you've set up your config to allow local connections.

To get all available channels, run `sync {host-url}`

To connect to channels, run `sync {host-url} channels`, ex. `sync localhost:10234 testlib1 testlib2`

## Notes

During the sync process, all requires are prefixed with `/` to make it search from the root of the drive. This will break requires that work relatively from the current file.

`sync.lua` is always minified, as the files it gets bundled with are fairly large (libdeflate.lua is 39.1kb, msgpack.lua is 10kb)

## Credits

msgpack.lua is from https://github.com/kieselsteini/msgpack

libdeflate.lua is from https://github.com/SafeteeWoW/LibDeflate
    - the provided LibDeflate is trimmed down to only include decompressdeflate to save on filesize

I forgot where base64.lua is from, but I remember it being from some roblox devforum thread.
