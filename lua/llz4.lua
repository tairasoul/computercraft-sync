-- from https://github.com/RiskoZoSlovenska/llz4/blob/main/llz4.lua
-- unnecessary aspects removed & changed to not use goto label, darklua's parser doesn't support goto & labels

local band, lshift, rshift = bit32.band, bit32.lshift, bit32.rshift

local string_byte = string.byte

local MIN_MATCH = 4 -- A sequence is 3 bytes, so it has to encode at least 4 to have any use
local MIN_LENGTH = 13
local MIN_TRAILING_LITERALS = 5
local MISS_COUNTER_BITS = 6 -- Lower values = the step is incremented sooner
local HASH_SHIFT = 32 - 16  -- 32 - # of bits in the hash
local MAX_DISTANCE = 0xFFFF -- Maximum offset that can fit into two bytes

local LIT_COUNT_BITS = 4
local LIT_COUNT_MASK = lshift(1, LIT_COUNT_BITS) - 1
local MATCH_LEN_BITS = 4
local MATCH_LEN_MASK = lshift(1, MATCH_LEN_BITS) - 1

local CHAR_MAP = {}
for i = 0, 255 do
	CHAR_MAP[i] = string.char(i)
end
local CHAR_0xFF = string.char(0xFF)


local function readU32LE(str, index)
	local a, b, c, d = string_byte(str, index, index + 3)
	return a + lshift(b, 8) + lshift(c, 16) + lshift(d, 24)
end

-- MARK: Compress
--[[=
	Compresses a string using the LZ4 block format.

	@param string data The string to compress.
	@param number? acceleration A positive integer, defaults to 1. Higher values
	  may increase the compression speed, especially on incompressible data, at
	  the cost of compression efficiency.
	@return string The compressed data as a string.
]]
local function compress(data, acceleration)
	assert(type(data) == "string", "bad argument #1 to 'compress' (string expected, got " .. type(data) .. ")")
	acceleration = acceleration or 1
	assert(type(acceleration) == "number", "bad argument #2 to 'compress' (number expected, got " .. type(acceleration) .. ")")
	assert(acceleration >= 1 and acceleration % 1 == 0, "acceleration must be an integer >= 1")

	local hashTable = {}
	local out, outNext = {}, 1

	local pos, dataLen = 1, #data -- 1-indexed
	local nextUnencodedPos = pos -- Sometimes called the "anchor" in other implementations

	if dataLen >= MIN_LENGTH then
		-- The lower MISS_COUNTER_BITS bits are the miss counter, upper bits are the step. The step
		-- starts at `acceleration` and increments every time the miss counter overflows.
		local stepAndMissCounterInit = lshift(acceleration, MISS_COUNTER_BITS)
		local stepAndMissCounter = stepAndMissCounterInit

		while pos + MIN_MATCH <= dataLen - MIN_TRAILING_LITERALS do
			local sequence = readU32LE(data, pos)
			local hash = rshift(sequence * 2654435761, HASH_SHIFT)
			-- ^ This is awfully simple for a hash function, but it's fast and seems to give pretty good results. The
			-- magic constant was taken from https://github.com/lz4/lz4/blob/836decd8a898475dcd21ed46768157f4420c9dd2/lib/lz4.c#L782

			-- Check and update match
			local matchPos = hashTable[hash]
			hashTable[hash] = pos

			-- Determine if there is a match in range
			if not matchPos or pos - matchPos > MAX_DISTANCE or readU32LE(data, matchPos) ~= sequence then
				pos = pos + rshift(stepAndMissCounter, MISS_COUNTER_BITS) -- Extract and add the step part
				stepAndMissCounter = stepAndMissCounter + 1
			else
				stepAndMissCounter = stepAndMissCounterInit

				-- Calculate literal count and offset
				local literalCount = pos - nextUnencodedPos
				local matchOffset = pos - matchPos

				-- Try to extend backwards
				while literalCount > 0 and matchPos > 0 and string_byte(data, pos - 1) == string_byte(data, matchPos - 1) do
					literalCount = literalCount - 1
					pos = pos - 1
					matchPos = matchPos - 1
				end

				-- Skip the 4 bytes we already matched
				pos = pos + MIN_MATCH
				matchPos = matchPos + MIN_MATCH

				-- Determine match length
				-- NOTE: matchLength does not include minMatch; it is added during decoding
				local matchLength = pos
				while pos <= dataLen - MIN_TRAILING_LITERALS and string_byte(data, pos) == string_byte(data, matchPos) do
					pos = pos + 1
					matchPos = matchPos + 1
				end
				matchLength = pos - matchLength

				-- Write token
				local literalCountHalf = (literalCount < LIT_COUNT_MASK) and literalCount or LIT_COUNT_MASK
				local matchLenHalf = (matchLength < MATCH_LEN_MASK) and matchLength or MATCH_LEN_MASK
				local token = lshift(literalCountHalf, MATCH_LEN_BITS) + matchLenHalf
				out[outNext] = CHAR_MAP[token]
				outNext = outNext + 1

				-- Write literal count
				local remaining = literalCount - LIT_COUNT_MASK
				while remaining >= 0xFF do
					out[outNext] = CHAR_0xFF
					outNext = outNext + 1
					remaining = remaining - 0xFF
				end
				if remaining >= 0 then
					out[outNext] = CHAR_MAP[remaining]
					outNext = outNext + 1
				end

				-- Write literals
				for i = 0, literalCount - 1 do
					out[outNext + i] = CHAR_MAP[string_byte(data, nextUnencodedPos + i)]
				end
				outNext = outNext + literalCount

				-- Write offset (little-endian)
				out[outNext    ] = CHAR_MAP[band(matchOffset, 0xFF)]
				out[outNext + 1] = CHAR_MAP[rshift(matchOffset, 8)]
				outNext = outNext + 2

				-- Write match length
				remaining = matchLength - MATCH_LEN_MASK
				while remaining >= 0xFF do
					out[outNext] = CHAR_0xFF
					outNext = outNext + 1
					remaining = remaining - 0xFF
				end
				if remaining >= 0 then
					out[outNext] = CHAR_MAP[remaining]
					outNext = outNext + 1
				end

				-- Move the anchor
				nextUnencodedPos = pos
			end
		end
	end

	-- Write remaining token (only literals, match length is 0)
	local literalCount = dataLen - nextUnencodedPos + 1
	local token = lshift((literalCount < LIT_COUNT_MASK) and literalCount or LIT_COUNT_MASK, MATCH_LEN_BITS)
	out[outNext] = CHAR_MAP[token]
	outNext = outNext + 1

	-- Write remaining literal count
	local remaining = literalCount - LIT_COUNT_MASK
	while remaining >= 0xFF do
		out[outNext] = CHAR_0xFF
		outNext = outNext + 1
		remaining = remaining - 0xFF
	end
	if remaining >= 0 then
		out[outNext] = CHAR_MAP[remaining]
		outNext = outNext + 1
	end

	-- Write remaining literals
	out[outNext] = string.sub(data, nextUnencodedPos)

	return table.concat(out)
end


-- MARK: Decompress
--[[=
	Decompresses a string that was compressed using the LZ4 block format. If any
	issue is encountered during decompression, this function will throw an
	error; call via `pcall()` when processing untrusted input.

	@param string data The string to decompress.
	@return string The decompressed string.
]]
local function decompress(data)
	assert(type(data) == "string", "bad argument #1 to 'decompress' (string expected, got " .. type(data) .. ")")

	local out, outNext = {}, 1

	local dataLen = #data
	local pos = 1 -- 1-indexed
	while pos <= dataLen do
		local token = string_byte(data, pos)
		pos = pos + 1

		-- Literals --
		local literalCount = rshift(token, MATCH_LEN_BITS)

		-- Read literal count
		if literalCount == LIT_COUNT_MASK then
			repeat
				local lenPart = string_byte(data, pos)
				pos = pos + 1
				literalCount = literalCount + lenPart
			until lenPart < 0xFF
		end

		-- Copy literals (if any)
		for i = 0, literalCount - 1 do
			out[outNext + i] = CHAR_MAP[string_byte(data, pos + i)]
		end
		outNext = outNext + literalCount
		pos = pos + literalCount

		if pos > dataLen then
			break -- This was the last sequence (which has no match part)
		end

		-- Match --
		local matchLength = band(token, MATCH_LEN_MASK)

		local offsetA, offsetB = string_byte(data, pos, pos + 1)
		local matchOffset = offsetA + lshift(offsetB, 8)
		pos = pos + 2

		-- Read match length
		if matchLength == MATCH_LEN_MASK then
			repeat
				local lenPart = string_byte(data, pos)
				pos = pos + 1
				matchLength = matchLength + lenPart
			until lenPart < 0xFF
		end

		matchLength = matchLength + MIN_MATCH

		-- Copy match
		for i = 0, matchLength - 1 do
			out[outNext + i] = out[outNext - matchOffset + i]
		end
		outNext = outNext + matchLength
	end

	return table.concat(out)
end


return {
	compress = compress,
	decompress = decompress,
}