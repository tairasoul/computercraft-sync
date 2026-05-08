local libDeflate = require("/cc-sync/libdeflate").libDeflate
local address = arg[1]
local print = print
local table_insert = table.insert
local string_unpack = string.unpack
local fs_exists = fs.exists
local fs_delete = fs.delete
local fs_open = fs.open
local fs_makeDir = fs.makeDir
local string_sub = string.sub
local os_date = os.date

local function split(input, delimiter)
  local result = {}
  for part in string.gmatch(input, "([^" .. delimiter .. "]+)") do
    table_insert(result, part)
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

if not arg[2] then 
  local request = http.get("http://" .. address)
  local raw = libDeflate:DecompressDeflate(request.readAll())
  textutils.pagedPrint("available channels:\n"..trim(raw))
  return
end

local channels = { select(2, unpack(arg)) }
local ws_addr = "ws://" .. address .. "/subscribe?channels=" .. table.concat(channels, ",")
print("connecting to address " .. ws_addr)
local ws, err = http.websocket(ws_addr)
if not ws then
  print("failed when connecting")
  print(err)
  return
end

local function decode(data)
  local ret = {}
  local len = #data
  local offset = 1
  while true do
    local tag = string_unpack(">I1", data, offset)
    offset = offset + 1
    if tag == 0 then
      local fp_len = string_unpack(">I4", data, offset)
      local fd_len = string_unpack(">I4", data, offset + 4)
      local fp = string_sub(data, offset, offset + 8 + fp_len - 1)
      offset = offset + 8 + fp_len
      local fd = string_sub(data, offset, offset + fd_len - 1)
      offset = offset + fd_len
      table_insert(ret, {fp = fp, fd = fd})
    elseif tag == 1 then
      local strings = {}
      local string_len = string_unpack(">I4", data, offset)
      offset = offset + 4
      for i = 1, string_len do
        local l = string_unpack(">I4", data, offset)
        table_insert(strings, string_sub(data, offset, offset + 4 + l - 1))
        offset = offset + 4 + l
      end
      table_insert(ret, {f = strings})
    elseif tag == 2 then
      local chunk_len = string_unpack(">I4", data, offset)
      table_insert(ret, {fd = string_sub(data, offset, offset + 4 + chunk_len - 1)})
      offset = offset + 4 + chunk_len
    end
    if offset > len then break end
  end
  return ret
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
    local data = decode(rdata)
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
    local f = fs_open(currentDir, "w+")
    f.write(data)
    f.close()
    return
  end
  if #dir > 2 then
	  for i = 2, #currentDir - 1 do
      if i == #dir then break end
      if not dir[i] then break end
		  currentDir = currentDir .. "/" .. dir[i]
		  if not fs_exists(currentDir) then 
		    fs_makeDir(currentDir)
		  end
	  end
  else
    if not fs_exists(currentDir) then fs_makeDir(currentDir) end
  end
  currentDir = currentDir .. "/" .. dir[#dir]
  local file = fs_open(currentDir, "w")
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
      fs_delete(folder)
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
  local f = fs_open(lastFile, "a")
  f.write(data.fd)
  f.close()
end

local function processData(data)
  if data.f ~= nil then
    print("[" .. os_date("%H:%M:%S") .. "] processing deletion sync request")
    for _,v in pairs(data.f) do
      fs_delete(v)
      walkUpTree(v)
    end
  elseif data.fp == nil then
    print("[" .. os_date("%H:%M:%S") .. "] processing chunked sync request")
    addPortion(data)
  elseif data.fp ~= nil then
    print("[" .. os_date("%H:%M:%S") .. "] processing data sync request")
    ensureFile(data.fp, data.fd)
    lastFile = data.fp
  end
end

local function log_p()
  print("[" .. os_date("%H:%M:%S") .. "] waiting for data")
end

log_p()

while true do
  local data, close = receive()
  if close then break end
  if data then
    for _,v in pairs(data) do
      processData(v)
    end
    os.queueEvent("channel_update", channels)
    log_p()
  end
end