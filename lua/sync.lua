local args = arg 
local msgpack = require("./msgpack")
local libDeflate = require("/cc-sync/libdeflate").libDeflate
local address = args[1]

local function split(input, delimiter)
  local result = {}
  for part in string.gmatch(input, "([^" .. delimiter .. "]+)") do
    table.insert(result, part)
  end
  return result
end

if not address then 
  print("sync.lua usage:")
  print("sync address channels")
  print("-----")
  print("sync address -- lists channels")
  print("sync address channels -- connects to a list of channels")
  print("example:")
  print("sync localhost:10234 lib1 lib2")
  return
end

local function trim(s) return s:match'^()%s*$' and '' or s:match'^%s*(.*%S)' end

if not args[2] then 
  local request = http.get("http://" .. address)
  local raw = libDeflate:DecompressDeflate(request.readAll())
  textutils.pagedPrint("available channels:\n"..trim(raw))
  return
end

local channels = { select(2, unpack(args)) }
local ws_addr = "ws://" .. address .. "/subscribe?channels=" .. table.concat(channels, ",")
print("connecting to address " .. ws_addr)
local ws, err = http.websocket(ws_addr)
if not ws then
  print("failed when connecting")
  print(err)
  return
end

local function receive() 
  local ev, ev1, ev2, ev3 = os.pullEventRaw()
  if ev == "websocket_closed" then
    return nil, true
  end
  if ev == "terminate" then
    ws.close()
    return nil, true
  end
  if ev ~= "websocket_message" then
    return nil, false
  end
  local recv, isBinary = ev2, ev3
  if not recv then print("websocket likely closed, ending program") return nil, true end
  if not isBinary then error("non-binary message received:\n"..recv) return nil, true end
  if recv then
    local rdata = libDeflate:DecompressDeflate(recv)
    local data = msgpack.decode(rdata)
    --local dfok, rdata = pcall(function()
    --  return libDeflate:DecompressDeflate(recv)
    --end)
    --if not dfok then return error("error inflating pako data") end
    --local ok, data = pcall(msgpack.decode, rdata)
    --if not ok then return error("error decoding msgpack data" .. data) end
    return data, false
  end
end

local function ensureFile(path, data)
  local dir = split(path, "/")
  local currentDir = dir[1]
  if #dir == 1 then
    local f = fs.open(currentDir, "w+")
    f.write(data)
    f.close()
    return
  end
  if #dir > 2 then
	  for i = 2, #currentDir - 1 do
      if i == #dir then break end
      if not dir[i] then break end
		  currentDir = currentDir .. "/" .. dir[i]
		  if not fs.exists(currentDir) then 
		    fs.makeDir(currentDir)
		  end
	  end
  else
    if not fs.exists(currentDir) then fs.makeDir(currentDir) end
  end
  currentDir = currentDir .. "/" .. dir[#dir]
  local file = fs.open(currentDir, "w")
  file.write(data)
  file.close()
end

local function walkUpTree(path)
  local path = split(path, "/")
  if #path == 1 then return end
  local currentPath = path[1]
  local function checkFolder(folder)
    print("checking folder " .. folder)
    local list = fs.list(folder)
    if #list == 0 then 
      print(folder .. " is empty, deleting")
      fs.delete(folder)
      return
    end
    print(folder .. " is not empty, checking children")
    for _,v in next, list do
      if fs.isDir(v) then checkFolder(folder .. "/" .. v) end
    end
  end
  checkFolder(currentPath)
end

local lastFile = ""

local function addPortion(data)
  local f = fs.open(lastFile, "a")
  f.write(data.fileData)
  f.close()
end

local function processData(data)
  print("processing " .. data.type .. " sync request")
  if data.type == "d" then
    for _,v in pairs(data.files) do
      fs.delete(v)
      walkUpTree(v)
    end
  elseif data.type == "c" then
    addPortion(data)
  else
    ensureFile(data.filePath, data.fileData)
    lastFile = data.filePath
  end
end

print("[" .. os.date("%H:%M:%S") .. "] waiting for data")

while true do
  local data, close = receive()
  if close then break end
  if data then
    for _,v in pairs(data) do
      processData(v)
    end
    print("[" .. os.date("%H:%M:%S") .. "] waiting for data")
  end
end