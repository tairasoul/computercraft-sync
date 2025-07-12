-- THIS FILE IS MODIFIED TO ONLY INCLUDE ESSENTIALS FOR SYNC.LUA

--[[--
LibDeflate 1.0.2-release <br>
Pure Lua compressor and decompressor with high compression ratio using
DEFLATE/zlib format.

@file LibDeflate.lua
@author Haoqian He (Github: SafeteeWoW; World of Warcraft: Safetyy-Illidan(US))
@copyright LibDeflate <2018-2021> Haoqian He
@license zlib License

This library is implemented according to the following specifications. <br>
Report a bug if LibDeflate is not fully compliant with those specs. <br>
Both compressors and decompressors have been implemented in the library.<br>
1. RFC1950: DEFLATE Compressed Data Format Specification version 1.3 <br>
https://tools.ietf.org/html/rfc1951 <br>
2. RFC1951: ZLIB Compressed Data Format Specification version 3.3 <br>
https://tools.ietf.org/html/rfc1950 <br>

This library requires Lua 5.1/5.2/5.3/5.4 interpreter or LuaJIT v2.0+. <br>
This library does not have any dependencies. <br>
<br>
This file "LibDeflate.lua" is the only source file of
the library. <br>
Submit suggestions or report bugs to
https://github.com/safeteeWow/LibDeflate/issues
]] --[[
zlib License

(C) 2018-2021 Haoqian He

This software is provided 'as-is', without any express or implied
warranty.  In no event will the authors be held liable for any damages
arising from the use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not
   claim that you wrote the original software. If you use this software
   in a product, an acknowledgment in the product documentation would be
   appreciated but is not required.
2. Altered source versions must be plainly marked as such, and must not be
   misrepresented as being the original software.
3. This notice may not be removed or altered from any source distribution.

License History:
1. GNU General Public License Version 3 in v1.0.0 and earlier versions.
2. GNU Lesser General Public License Version 3 in v1.0.1
3. the zlib License since v1.0.2

Credits and Disclaimer:
This library rewrites the code from the algorithm
and the ideas of the following projects,
and uses their code to help to test the correctness of this library,
but their code is not included directly in the library itself.
Their original licenses shall be comply when used.

1. zlib, by Jean-loup Gailly (compression) and Mark Adler (decompression).
	http://www.zlib.net/
	Licensed under zlib License. http://www.zlib.net/zlib_license.html
	For the compression algorithm.
2. puff, by Mark Adler. https://github.com/madler/zlib/tree/master/contrib/puff
	Licensed under zlib License. http://www.zlib.net/zlib_license.html
	For the decompression algorithm.
3. LibCompress, by jjsheets and Galmok of European Stormrage (Horde)
	https://www.wowace.com/projects/libcompress
	Licensed under GPLv2.
	https://www.gnu.org/licenses/old-licenses/gpl-2.0.html
	For the code to create customized codec.
4. WeakAuras2,
	https://github.com/WeakAuras/WeakAuras2
	Licensed under GPLv2.
	For the 6bit encoding and decoding.
]] --[[
	Curseforge auto-packaging replacements:

	Project Date: @project-date-iso@
	Project Hash: @project-hash@
	Project Version: @project-version@
--]] local LibDeflate

do
  -- Semantic version. all lowercase.
  -- Suffix can be alpha1, alpha2, beta1, beta2, rc1, rc2, etc.
  -- NOTE: Two version numbers needs to modify.
  -- 1. On the top of LibDeflate.lua
  -- 2. _VERSION
  -- 3. _MINOR

  -- version to store the official version of LibDeflate
  local _VERSION = "1.0.2-release"

  -- When MAJOR is changed, I should name it as LibDeflate2
  local _MAJOR = "LibDeflate"

  -- Update this whenever a new version, for LibStub version registration.
  -- 0 : v0.x
  -- 1 : v1.0.0
  -- 2 : v1.0.1
  -- 3 : v1.0.2
  local _MINOR = 3

  local _COPYRIGHT = "LibDeflate " .. _VERSION ..
                       " Copyright (C) 2018-2021 Haoqian He." ..
                       " Licensed under the zlib License"
  LibDeflate = {}

  LibDeflate._VERSION = _VERSION
  LibDeflate._MAJOR = _MAJOR
  LibDeflate._MINOR = _MINOR
  LibDeflate._COPYRIGHT = _COPYRIGHT
end

-- localize Lua api for faster access.
local assert = assert
local error = error
local pairs = pairs
local string_byte = string.byte
local string_char = string.char
local string_find = string.find
local string_gsub = string.gsub
local string_sub = string.sub
local table_concat = table.concat
local table_sort = table.sort
local tostring = tostring
local type = type

-- Converts i to 2^i, (0<=i<=32)
-- This is used to implement bit left shift and bit right shift.
-- "x >> y" in C:   "(x-x%_pow2[y])/_pow2[y]" in Lua
-- "x << y" in C:   "x*_pow2[y]" in Lua
local _pow2 = {}

-- Converts any byte to a character, (0<=byte<=255)
local _byte_to_char = {}

-- _reverseBitsTbl[len][val] stores the bit reverse of
-- the number with bit length "len" and value "val"
-- For example, decimal number 6 with bits length 5 is binary 00110
-- It's reverse is binary 01100,
-- which is decimal 12 and 12 == _reverseBitsTbl[5][6]
-- 1<=len<=9, 0<=val<=2^len-1
-- The reason for 1<=len<=9 is that the max of min bitlen of huffman code
-- of a huffman alphabet is 9?
local _reverse_bits_tbl = {}

-- Convert a LZ77 length (3<=len<=258) to
-- a deflate literal/LZ77_length code (257<=code<=285)
local _length_to_deflate_code = {}

-- convert a LZ77 length (3<=len<=258) to
-- a deflate literal/LZ77_length code extra bits.
local _length_to_deflate_extra_bits = {}

-- Convert a LZ77 length (3<=len<=258) to
-- a deflate literal/LZ77_length code extra bit length.
local _length_to_deflate_extra_bitlen = {}

-- Convert a small LZ77 distance (1<=dist<=256) to a deflate code.
local _dist256_to_deflate_code = {}

-- Convert a small LZ77 distance (1<=dist<=256) to
-- a deflate distance code extra bits.
local _dist256_to_deflate_extra_bits = {}

-- Convert a small LZ77 distance (1<=dist<=256) to
-- a deflate distance code extra bit length.
local _dist256_to_deflate_extra_bitlen = {}

-- Convert a literal/LZ77_length deflate code to LZ77 base length
-- The key of the table is (code - 256), 257<=code<=285
local _literal_deflate_code_to_base_len =
  {
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67,
    83, 99, 115, 131, 163, 195, 227, 258
  }

-- Convert a literal/LZ77_length deflate code to base LZ77 length extra bits
-- The key of the table is (code - 256), 257<=code<=285
local _literal_deflate_code_to_extra_bitlen =
  {
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5,
    5, 5, 5, 0
  }

-- Convert a distance deflate code to base LZ77 distance. (0<=code<=29)
local _dist_deflate_code_to_base_dist = {
  [0] = 1,
  2,
  3,
  4,
  5,
  7,
  9,
  13,
  17,
  25,
  33,
  49,
  65,
  97,
  129,
  193,
  257,
  385,
  513,
  769,
  1025,
  1537,
  2049,
  3073,
  4097,
  6145,
  8193,
  12289,
  16385,
  24577
}

-- Convert a distance deflate code to LZ77 bits length. (0<=code<=29)
local _dist_deflate_code_to_extra_bitlen =
  {
    [0] = 0,
    0,
    0,
    0,
    1,
    1,
    2,
    2,
    3,
    3,
    4,
    4,
    5,
    5,
    6,
    6,
    7,
    7,
    8,
    8,
    9,
    9,
    10,
    10,
    11,
    11,
    12,
    12,
    13,
    13
  }

-- The code order of the first huffman header in the dynamic deflate block.
-- See the page 12 of RFC1951
local _rle_codes_huffman_bitlen_order = {
  16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
}

-- The following tables are used by fixed deflate block.
-- The value of these tables are assigned at the bottom of the source.

-- The huffman code of the literal/LZ77_length deflate codes,
-- in fixed deflate block.
local _fix_block_literal_huffman_code

-- Convert huffman code of the literal/LZ77_length to deflate codes,
-- in fixed deflate block.
local _fix_block_literal_huffman_to_deflate_code

-- The bit length of the huffman code of literal/LZ77_length deflate codes,
-- in fixed deflate block.
local _fix_block_literal_huffman_bitlen

-- The count of each bit length of the literal/LZ77_length deflate codes,
-- in fixed deflate block.
local _fix_block_literal_huffman_bitlen_count

-- The huffman code of the distance deflate codes,
-- in fixed deflate block.
local _fix_block_dist_huffman_code

-- Convert huffman code of the distance to deflate codes,
-- in fixed deflate block.
local _fix_block_dist_huffman_to_deflate_code

-- The bit length of the huffman code of the distance deflate codes,
-- in fixed deflate block.
local _fix_block_dist_huffman_bitlen

-- The count of each bit length of the huffman code of
-- the distance deflate codes,
-- in fixed deflate block.
local _fix_block_dist_huffman_bitlen_count

for i = 0, 255 do _byte_to_char[i] = string_char(i) end

do
  local pow = 1
  for i = 0, 32 do
    _pow2[i] = pow
    pow = pow * 2
  end
end

for i = 1, 9 do
  _reverse_bits_tbl[i] = {}
  for j = 0, _pow2[i + 1] - 1 do
    local reverse = 0
    local value = j
    for _ = 1, i do
      -- The following line is equivalent to "res | (code %2)" in C.
      reverse = reverse - reverse % 2 +
                  (((reverse % 2 == 1) or (value % 2) == 1) and 1 or 0)
      value = (value - value % 2) / 2
      reverse = reverse * 2
    end
    _reverse_bits_tbl[i][j] = (reverse - reverse % 2) / 2
  end
end

-- The source code is written according to the pattern in the numbers
-- in RFC1951 Page10.
do
  local a = 18
  local b = 16
  local c = 265
  local bitlen = 1
  for len = 3, 258 do
    if len <= 10 then
      _length_to_deflate_code[len] = len + 254
      _length_to_deflate_extra_bitlen[len] = 0
    elseif len == 258 then
      _length_to_deflate_code[len] = 285
      _length_to_deflate_extra_bitlen[len] = 0
    else
      if len > a then
        a = a + b
        b = b * 2
        c = c + 4
        bitlen = bitlen + 1
      end
      local t = len - a - 1 + b / 2
      _length_to_deflate_code[len] = (t - (t % (b / 8))) / (b / 8) + c
      _length_to_deflate_extra_bitlen[len] = bitlen
      _length_to_deflate_extra_bits[len] = t % (b / 8)
    end
  end
end

-- The source code is written according to the pattern in the numbers
-- in RFC1951 Page11.
do
  _dist256_to_deflate_code[1] = 0
  _dist256_to_deflate_code[2] = 1
  _dist256_to_deflate_extra_bitlen[1] = 0
  _dist256_to_deflate_extra_bitlen[2] = 0

  local a = 3
  local b = 4
  local code = 2
  local bitlen = 0
  for dist = 3, 256 do
    if dist > b then
      a = a * 2
      b = b * 2
      code = code + 2
      bitlen = bitlen + 1
    end
    _dist256_to_deflate_code[dist] = (dist <= a) and code or (code + 1)
    _dist256_to_deflate_extra_bitlen[dist] = (bitlen < 0) and 0 or bitlen
    if b >= 8 then
      _dist256_to_deflate_extra_bits[dist] = (dist - b / 2 - 1) % (b / 4)
    end
  end
end

-- Check if the dictionary is valid.
-- @param dictionary The preset dictionary for compression and decompression.
-- @return true if valid, false if not valid.
-- @return if not valid, the error message.
local function IsValidDictionary(dictionary)
  if type(dictionary) ~= "table" then
    return false,
           ("'dictionary' - table expected got '%s'."):format(type(dictionary))
  end
  if type(dictionary.adler32) ~= "number" or type(dictionary.string_table) ~=
    "table" or type(dictionary.strlen) ~= "number" or dictionary.strlen <= 0 or
    dictionary.strlen > 32768 or dictionary.strlen ~= #dictionary.string_table or
    type(dictionary.hash_tables) ~= "table" then
    return false,
           ("'dictionary' - corrupted dictionary."):format(type(dictionary))
  end
  return true, ""
end

--[[
	key of the configuration table is the compression level,
	and its value stores the compression setting.
	These numbers come from zlib source code.

	Higher compression level usually means better compression.
	(Because LibDeflate uses a simplified version of zlib algorithm,
	there is no guarantee that higher compression level does not create
	bigger file than lower level, but I can say it's 99% likely)

	Be careful with the high compression level. This is a pure lua
	implementation compressor/decompressor, which is significant slower than
	a C/C++ equivalant compressor/decompressor. Very high compression level
	costs significant more CPU time, and usually compression size won't be
	significant smaller when you increase compression level by 1, when the
	level is already very high. Benchmark yourself if you can afford it.

	See also https://github.com/madler/zlib/blob/master/doc/algorithm.txt,
	https://github.com/madler/zlib/blob/master/deflate.c for more information.

	The meaning of each field:
	@field 1 use_lazy_evaluation:
		true/false. Whether the program uses lazy evaluation.
		See what is "lazy evaluation" in the link above.
		lazy_evaluation improves ratio, but relatively slow.
	@field 2 good_prev_length:
		Only effective if lazy is set, Only use 1/4 of max_chain,
		if prev length of lazy match is above this.
	@field 3 max_insert_length/max_lazy_match:
		If not using lazy evaluation,
		insert new strings in the hash table only if the match length is not
		greater than this length.
		If using lazy evaluation, only continue lazy evaluation,
		if previous match length is strictly smaller than this value.
	@field 4 nice_length:
		Number. Don't continue to go down the hash chain,
		if match length is above this.
	@field 5 max_chain:
		Number. The maximum number of hash chains we look.
--]]
local _compression_level_configs = {
  [0] = {false, nil, 0, 0, 0}, -- level 0, no compression
  [1] = {false, nil, 4, 8, 4}, -- level 1, similar to zlib level 1
  [2] = {false, nil, 5, 18, 8}, -- level 2, similar to zlib level 2
  [3] = {false, nil, 6, 32, 32}, -- level 3, similar to zlib level 3
  [4] = {true, 4, 4, 16, 16}, -- level 4, similar to zlib level 4
  [5] = {true, 8, 16, 32, 32}, -- level 5, similar to zlib level 5
  [6] = {true, 8, 16, 128, 128}, -- level 6, similar to zlib level 6
  [7] = {true, 8, 32, 128, 256}, -- (SLOW) level 7, similar to zlib level 7
  [8] = {true, 32, 128, 258, 1024}, -- (SLOW) level 8,similar to zlib level 8
  [9] = {true, 32, 258, 258, 4096}
  -- (VERY SLOW) level 9, similar to zlib level 9
}

-- Check if the compression/decompression arguments is valid
-- @param str The input string.
-- @param check_dictionary if true, check if dictionary is valid.
-- @param dictionary The preset dictionary for compression and decompression.
-- @param check_configs if true, check if config is valid.
-- @param configs The compression configuration table
-- @return true if valid, false if not valid.
-- @return if not valid, the error message.
local function IsValidArguments(str, check_dictionary, dictionary,
                                check_configs, configs)

  if type(str) ~= "string" then
    return false, ("'str' - string expected got '%s'."):format(type(str))
  end
  if check_dictionary then
    local dict_valid, dict_err = IsValidDictionary(dictionary)
    if not dict_valid then return false, dict_err end
  end
  if check_configs then
    local type_configs = type(configs)
    if type_configs ~= "nil" and type_configs ~= "table" then
      return false, ("'configs' - nil or table expected got '%s'."):format(
               type(configs))
    end
    if type_configs == "table" then
      for k, v in pairs(configs) do
        if k ~= "level" and k ~= "strategy" then
          return false,
                 ("'configs' - unsupported table key in the configs: '%s'."):format(
                   k)
        elseif k == "level" and not _compression_level_configs[v] then
          return false,
                 ("'configs' - unsupported 'level': %s."):format(tostring(v))
        elseif k == "strategy" and v ~= "fixed" and v ~= "huffman_only" and v ~=
          "dynamic" then
          -- random_block_type is for testing purpose
          return false, ("'configs' - unsupported 'strategy': '%s'."):format(
                   tostring(v))
        end
      end
    end
  end
  return true, ""
end

--[[ --------------------------------------------------------------------------
	Decompress code
--]] --------------------------------------------------------------------------

--[[
	Create a reader to easily reader stuffs as the unit of bits.
	Return values:
	1. ReadBits(bitlen)
	2. ReadBytes(bytelen, buffer, buffer_size)
	3. Decode(huffman_bitlen_count, huffman_symbol, min_bitlen)
	4. ReaderBitlenLeft()
	5. SkipToByteBoundary()
--]]
local function CreateReader(input_string)
  local input = input_string
  local input_strlen = #input_string
  local input_next_byte_pos = 1
  local cache_bitlen = 0
  local cache = 0

  -- Read some bits.
  -- To improve speed, this function does not
  -- check if the input has been exhausted.
  -- Use ReaderBitlenLeft() < 0 to check it.
  -- @param bitlen the number of bits to read
  -- @return the data is read.
  local function ReadBits(bitlen)
    local rshift_mask = _pow2[bitlen]
    local code
    if bitlen <= cache_bitlen then
      code = cache % rshift_mask
      cache = (cache - code) / rshift_mask
      cache_bitlen = cache_bitlen - bitlen
    else -- Whether input has been exhausted is not checked.
      local lshift_mask = _pow2[cache_bitlen]
      local byte1, byte2, byte3, byte4 =
        string_byte(input, input_next_byte_pos, input_next_byte_pos + 3)
      -- This requires lua number to be at least double ()
      cache = cache +
                ((byte1 or 0) + (byte2 or 0) * 256 + (byte3 or 0) * 65536 +
                  (byte4 or 0) * 16777216) * lshift_mask
      input_next_byte_pos = input_next_byte_pos + 4
      cache_bitlen = cache_bitlen + 32 - bitlen
      code = cache % rshift_mask
      cache = (cache - code) / rshift_mask
    end
    return code
  end

  -- Read some bytes from the reader.
  -- Assume reader is on the byte boundary.
  -- @param bytelen The number of bytes to be read.
  -- @param buffer The byte read will be stored into this buffer.
  -- @param buffer_size The buffer will be modified starting from
  --	buffer[buffer_size+1], ending at buffer[buffer_size+bytelen-1]
  -- @return the new buffer_size
  local function ReadBytes(bytelen, buffer, buffer_size)
    assert(cache_bitlen % 8 == 0)

    local byte_from_cache =
      (cache_bitlen / 8 < bytelen) and (cache_bitlen / 8) or bytelen
    for _ = 1, byte_from_cache do
      local byte = cache % 256
      buffer_size = buffer_size + 1
      buffer[buffer_size] = string_char(byte)
      cache = (cache - byte) / 256
    end
    cache_bitlen = cache_bitlen - byte_from_cache * 8
    bytelen = bytelen - byte_from_cache
    if (input_strlen - input_next_byte_pos - bytelen + 1) * 8 + cache_bitlen < 0 then
      return -1 -- out of input
    end
    for i = input_next_byte_pos, input_next_byte_pos + bytelen - 1 do
      buffer_size = buffer_size + 1
      buffer[buffer_size] = string_sub(input, i, i)
    end

    input_next_byte_pos = input_next_byte_pos + bytelen
    return buffer_size
  end

  -- Decode huffman code
  -- To improve speed, this function does not check
  -- if the input has been exhausted.
  -- Use ReaderBitlenLeft() < 0 to check it.
  -- Credits for Mark Adler. This code is from puff:Decode()
  -- @see puff:Decode(...)
  -- @param huffman_bitlen_count
  -- @param huffman_symbol
  -- @param min_bitlen The minimum huffman bit length of all symbols
  -- @return The decoded deflate code.
  --	Negative value is returned if decoding fails.
  local function Decode(huffman_bitlen_counts, huffman_symbols, min_bitlen)
    local code = 0
    local first = 0
    local index = 0
    local count
    if min_bitlen > 0 then
      if cache_bitlen < 15 and input then
        local lshift_mask = _pow2[cache_bitlen]
        local byte1, byte2, byte3, byte4 =
          string_byte(input, input_next_byte_pos, input_next_byte_pos + 3)
        -- This requires lua number to be at least double ()
        cache = cache +
                  ((byte1 or 0) + (byte2 or 0) * 256 + (byte3 or 0) * 65536 +
                    (byte4 or 0) * 16777216) * lshift_mask
        input_next_byte_pos = input_next_byte_pos + 4
        cache_bitlen = cache_bitlen + 32
      end

      local rshift_mask = _pow2[min_bitlen]
      cache_bitlen = cache_bitlen - min_bitlen
      code = cache % rshift_mask
      cache = (cache - code) / rshift_mask
      -- Reverse the bits
      code = _reverse_bits_tbl[min_bitlen][code]

      count = huffman_bitlen_counts[min_bitlen]
      if code < count then return huffman_symbols[code] end
      index = count
      first = count * 2
      code = code * 2
    end

    for bitlen = min_bitlen + 1, 15 do
      local bit
      bit = cache % 2
      cache = (cache - bit) / 2
      cache_bitlen = cache_bitlen - 1

      code = (bit == 1) and (code + 1 - code % 2) or code
      count = huffman_bitlen_counts[bitlen] or 0
      local diff = code - first
      if diff < count then return huffman_symbols[index + diff] end
      index = index + count
      first = first + count
      first = first * 2
      code = code * 2
    end
    -- invalid literal/length or distance code
    -- in fixed or dynamic block (run out of code)
    return -10
  end

  local function ReaderBitlenLeft()
    return (input_strlen - input_next_byte_pos + 1) * 8 + cache_bitlen
  end

  local function SkipToByteBoundary()
    local skipped_bitlen = cache_bitlen % 8
    local rshift_mask = _pow2[skipped_bitlen]
    cache_bitlen = cache_bitlen - skipped_bitlen
    cache = (cache - cache % rshift_mask) / rshift_mask
  end

  return ReadBits, ReadBytes, Decode, ReaderBitlenLeft, SkipToByteBoundary
end

-- Create a deflate state, so I can pass in less arguments to functions.
-- @param str the whole string to be decompressed.
-- @param dictionary The preset dictionary. nil if not provided.
--		This dictionary should be produced by LibDeflate:CreateDictionary(str)
-- @return The decomrpess state.
local function CreateDecompressState(str, dictionary)
  local ReadBits, ReadBytes, Decode, ReaderBitlenLeft, SkipToByteBoundary =
    CreateReader(str)
  local state = {
    ReadBits = ReadBits,
    ReadBytes = ReadBytes,
    Decode = Decode,
    ReaderBitlenLeft = ReaderBitlenLeft,
    SkipToByteBoundary = SkipToByteBoundary,
    buffer_size = 0,
    buffer = {},
    result_buffer = {},
    dictionary = dictionary
  }
  return state
end

-- Get the stuffs needed to decode huffman codes
-- @see puff.c:construct(...)
-- @param huffman_bitlen The huffman bit length of the huffman codes.
-- @param max_symbol The maximum symbol
-- @param max_bitlen The min huffman bit length of all codes
-- @return zero or positive for success, negative for failure.
-- @return The count of each huffman bit length.
-- @return A table to convert huffman codes to deflate codes.
-- @return The minimum huffman bit length.
local function GetHuffmanForDecode(huffman_bitlens, max_symbol, max_bitlen)
  local huffman_bitlen_counts = {}
  local min_bitlen = max_bitlen
  for symbol = 0, max_symbol do
    local bitlen = huffman_bitlens[symbol] or 0
    min_bitlen = (bitlen > 0 and bitlen < min_bitlen) and bitlen or min_bitlen
    huffman_bitlen_counts[bitlen] = (huffman_bitlen_counts[bitlen] or 0) + 1
  end

  if huffman_bitlen_counts[0] == max_symbol + 1 then -- No Codes
    return 0, huffman_bitlen_counts, {}, 0 -- Complete, but decode will fail
  end

  local left = 1
  for len = 1, max_bitlen do
    left = left * 2
    left = left - (huffman_bitlen_counts[len] or 0)
    if left < 0 then
      return left -- Over-subscribed, return negative
    end
  end

  -- Generate offsets info symbol table for each length for sorting
  local offsets = {}
  offsets[1] = 0
  for len = 1, max_bitlen - 1 do
    offsets[len + 1] = offsets[len] + (huffman_bitlen_counts[len] or 0)
  end

  local huffman_symbols = {}
  for symbol = 0, max_symbol do
    local bitlen = huffman_bitlens[symbol] or 0
    if bitlen ~= 0 then
      local offset = offsets[bitlen]
      huffman_symbols[offset] = symbol
      offsets[bitlen] = offsets[bitlen] + 1
    end
  end

  -- Return zero for complete set, positive for incomplete set.
  return left, huffman_bitlen_counts, huffman_symbols, min_bitlen
end

-- Decode a fixed or dynamic huffman blocks, excluding last block identifier
-- and block type identifer.
-- @see puff.c:codes()
-- @param state decompression state that will be modified by this function.
--	@see CreateDecompressState
-- @param ... Read the source code
-- @return 0 on success, other value on failure.
local function DecodeUntilEndOfBlock(state, lcodes_huffman_bitlens,
                                     lcodes_huffman_symbols,
                                     lcodes_huffman_min_bitlen,
                                     dcodes_huffman_bitlens,
                                     dcodes_huffman_symbols,
                                     dcodes_huffman_min_bitlen)
  local buffer, buffer_size, ReadBits, Decode, ReaderBitlenLeft, result_buffer =
    state.buffer, state.buffer_size, state.ReadBits, state.Decode,
    state.ReaderBitlenLeft, state.result_buffer
  local dictionary = state.dictionary
  local dict_string_table
  local dict_strlen

  local buffer_end = 1
  if dictionary and not buffer[0] then
    -- If there is a dictionary, copy the last 258 bytes into
    -- the string_table to make the copy in the main loop quicker.
    -- This is done only once per decompression.
    dict_string_table = dictionary.string_table
    dict_strlen = dictionary.strlen
    buffer_end = -dict_strlen + 1
    for i = 0, (-dict_strlen + 1) < -257 and -257 or (-dict_strlen + 1), -1 do
      buffer[i] = _byte_to_char[dict_string_table[dict_strlen + i]]
    end
  end

  repeat
    local symbol = Decode(lcodes_huffman_bitlens, lcodes_huffman_symbols,
                          lcodes_huffman_min_bitlen)
    if symbol < 0 or symbol > 285 then
      -- invalid literal/length or distance code in fixed or dynamic block
      return -10
    elseif symbol < 256 then -- Literal
      buffer_size = buffer_size + 1
      buffer[buffer_size] = _byte_to_char[symbol]
    elseif symbol > 256 then -- Length code
      symbol = symbol - 256
      local bitlen = _literal_deflate_code_to_base_len[symbol]
      bitlen = (symbol >= 8) and
                 (bitlen +
                   ReadBits(_literal_deflate_code_to_extra_bitlen[symbol])) or
                 bitlen
      symbol = Decode(dcodes_huffman_bitlens, dcodes_huffman_symbols,
                      dcodes_huffman_min_bitlen)
      if symbol < 0 or symbol > 29 then
        -- invalid literal/length or distance code in fixed or dynamic block
        return -10
      end
      local dist = _dist_deflate_code_to_base_dist[symbol]
      dist = (dist > 4) and
               (dist + ReadBits(_dist_deflate_code_to_extra_bitlen[symbol])) or
               dist

      local char_buffer_index = buffer_size - dist + 1
      if char_buffer_index < buffer_end then
        -- distance is too far back in fixed or dynamic block
        return -11
      end
      if char_buffer_index >= -257 then
        for _ = 1, bitlen do
          buffer_size = buffer_size + 1
          buffer[buffer_size] = buffer[char_buffer_index]
          char_buffer_index = char_buffer_index + 1
        end
      else
        char_buffer_index = dict_strlen + char_buffer_index
        for _ = 1, bitlen do
          buffer_size = buffer_size + 1
          buffer[buffer_size] =
            _byte_to_char[dict_string_table[char_buffer_index]]
          char_buffer_index = char_buffer_index + 1
        end
      end
    end

    if ReaderBitlenLeft() < 0 then
      return 2 -- available inflate data did not terminate
    end

    if buffer_size >= 65536 then
      result_buffer[#result_buffer + 1] = table_concat(buffer, "", 1, 32768)
      for i = 32769, buffer_size do buffer[i - 32768] = buffer[i] end
      buffer_size = buffer_size - 32768
      buffer[buffer_size + 1] = nil
      -- NOTE: buffer[32769..end] and buffer[-257..0] are not cleared.
      -- This is why "buffer_size" variable is needed.
    end
  until symbol == 256

  state.buffer_size = buffer_size

  return 0
end

-- Decompress a store block
-- @param state decompression state that will be modified by this function.
-- @return 0 if succeeds, other value if fails.
local function DecompressStoreBlock(state)
  local buffer, buffer_size, ReadBits, ReadBytes, ReaderBitlenLeft,
        SkipToByteBoundary, result_buffer = state.buffer, state.buffer_size,
                                            state.ReadBits, state.ReadBytes,
                                            state.ReaderBitlenLeft,
                                            state.SkipToByteBoundary,
                                            state.result_buffer

  SkipToByteBoundary()
  local bytelen = ReadBits(16)
  if ReaderBitlenLeft() < 0 then
    return 2 -- available inflate data did not terminate
  end
  local bytelenComp = ReadBits(16)
  if ReaderBitlenLeft() < 0 then
    return 2 -- available inflate data did not terminate
  end

  if bytelen % 256 + bytelenComp % 256 ~= 255 then
    return -2 -- Not one's complement
  end
  if (bytelen - bytelen % 256) / 256 + (bytelenComp - bytelenComp % 256) / 256 ~=
    255 then
    return -2 -- Not one's complement
  end

  -- Note that ReadBytes will skip to the next byte boundary first.
  buffer_size = ReadBytes(bytelen, buffer, buffer_size)
  if buffer_size < 0 then
    return 2 -- available inflate data did not terminate
  end

  -- memory clean up when there are enough bytes in the buffer.
  if buffer_size >= 65536 then
    result_buffer[#result_buffer + 1] = table_concat(buffer, "", 1, 32768)
    for i = 32769, buffer_size do buffer[i - 32768] = buffer[i] end
    buffer_size = buffer_size - 32768
    buffer[buffer_size + 1] = nil
  end
  state.buffer_size = buffer_size
  return 0
end

-- Decompress a fixed block
-- @param state decompression state that will be modified by this function.
-- @return 0 if succeeds other value if fails.
local function DecompressFixBlock(state)
  return DecodeUntilEndOfBlock(state, _fix_block_literal_huffman_bitlen_count,
                               _fix_block_literal_huffman_to_deflate_code, 7,
                               _fix_block_dist_huffman_bitlen_count,
                               _fix_block_dist_huffman_to_deflate_code, 5)
end

-- Decompress a dynamic block
-- @param state decompression state that will be modified by this function.
-- @return 0 if success, other value if fails.
local function DecompressDynamicBlock(state)
  local ReadBits, Decode = state.ReadBits, state.Decode
  local nlen = ReadBits(5) + 257
  local ndist = ReadBits(5) + 1
  local ncode = ReadBits(4) + 4
  if nlen > 286 or ndist > 30 then
    -- dynamic block code description: too many length or distance codes
    return -3
  end

  local rle_codes_huffman_bitlens = {}

  for i = 1, ncode do
    rle_codes_huffman_bitlens[_rle_codes_huffman_bitlen_order[i]] = ReadBits(3)
  end

  local rle_codes_err, rle_codes_huffman_bitlen_counts,
        rle_codes_huffman_symbols, rle_codes_huffman_min_bitlen =
    GetHuffmanForDecode(rle_codes_huffman_bitlens, 18, 7)
  if rle_codes_err ~= 0 then -- Require complete code set here
    -- dynamic block code description: code lengths codes incomplete
    return -4
  end

  local lcodes_huffman_bitlens = {}
  local dcodes_huffman_bitlens = {}
  -- Read length/literal and distance code length tables
  local index = 0
  while index < nlen + ndist do
    local symbol -- Decoded value
    local bitlen -- Last length to repeat

    symbol = Decode(rle_codes_huffman_bitlen_counts, rle_codes_huffman_symbols,
                    rle_codes_huffman_min_bitlen)

    if symbol < 0 then
      return symbol -- Invalid symbol
    elseif symbol < 16 then
      if index < nlen then
        lcodes_huffman_bitlens[index] = symbol
      else
        dcodes_huffman_bitlens[index - nlen] = symbol
      end
      index = index + 1
    else
      bitlen = 0
      if symbol == 16 then
        if index == 0 then
          -- dynamic block code description: repeat lengths
          -- with no first length
          return -5
        end
        if index - 1 < nlen then
          bitlen = lcodes_huffman_bitlens[index - 1]
        else
          bitlen = dcodes_huffman_bitlens[index - nlen - 1]
        end
        symbol = 3 + ReadBits(2)
      elseif symbol == 17 then -- Repeat zero 3..10 times
        symbol = 3 + ReadBits(3)
      else -- == 18, repeat zero 11.138 times
        symbol = 11 + ReadBits(7)
      end
      if index + symbol > nlen + ndist then
        -- dynamic block code description:
        -- repeat more than specified lengths
        return -6
      end
      while symbol > 0 do -- Repeat last or zero symbol times
        symbol = symbol - 1
        if index < nlen then
          lcodes_huffman_bitlens[index] = bitlen
        else
          dcodes_huffman_bitlens[index - nlen] = bitlen
        end
        index = index + 1
      end
    end
  end

  if (lcodes_huffman_bitlens[256] or 0) == 0 then
    -- dynamic block code description: missing end-of-block code
    return -9
  end

  local lcodes_err, lcodes_huffman_bitlen_counts, lcodes_huffman_symbols,
        lcodes_huffman_min_bitlen = GetHuffmanForDecode(lcodes_huffman_bitlens,
                                                        nlen - 1, 15)
  -- dynamic block code description: invalid literal/length code lengths,
  -- Incomplete code ok only for single length 1 code
  if (lcodes_err ~= 0 and
    (lcodes_err < 0 or nlen ~= (lcodes_huffman_bitlen_counts[0] or 0) +
      (lcodes_huffman_bitlen_counts[1] or 0))) then return -7 end

  local dcodes_err, dcodes_huffman_bitlen_counts, dcodes_huffman_symbols,
        dcodes_huffman_min_bitlen = GetHuffmanForDecode(dcodes_huffman_bitlens,
                                                        ndist - 1, 15)
  -- dynamic block code description: invalid distance code lengths,
  -- Incomplete code ok only for single length 1 code
  if (dcodes_err ~= 0 and
    (dcodes_err < 0 or ndist ~= (dcodes_huffman_bitlen_counts[0] or 0) +
      (dcodes_huffman_bitlen_counts[1] or 0))) then return -8 end

  -- Build buffman table for literal/length codes
  return DecodeUntilEndOfBlock(state, lcodes_huffman_bitlen_counts,
                               lcodes_huffman_symbols,
                               lcodes_huffman_min_bitlen,
                               dcodes_huffman_bitlen_counts,
                               dcodes_huffman_symbols, dcodes_huffman_min_bitlen)
end

-- Decompress a deflate stream
-- @param state: a decompression state
-- @return the decompressed string if succeeds. nil if fails.
local function Inflate(state)
  local ReadBits = state.ReadBits

  local is_last_block
  while not is_last_block do
    is_last_block = (ReadBits(1) == 1)
    local block_type = ReadBits(2)
    local status
    if block_type == 0 then
      status = DecompressStoreBlock(state)
    elseif block_type == 1 then
      status = DecompressFixBlock(state)
    elseif block_type == 2 then
      status = DecompressDynamicBlock(state)
    else
      return nil, -1 -- invalid block type (type == 3)
    end
    if status ~= 0 then return nil, status end
  end

  state.result_buffer[#state.result_buffer + 1] =
    table_concat(state.buffer, "", 1, state.buffer_size)
  local result = table_concat(state.result_buffer)
  return result
end

-- @see LibDeflate:DecompressDeflate(str)
-- @see LibDeflate:DecompressDeflateWithDict(str, dictionary)
local function DecompressDeflateInternal(str, dictionary)
  local state = CreateDecompressState(str, dictionary)
  local result, status = Inflate(state)
  if not result then return nil, status end

  local bitlen_left = state.ReaderBitlenLeft()
  local bytelen_left = (bitlen_left - bitlen_left % 8) / 8
  return result, bytelen_left
end

--- Decompress a raw deflate compressed data.
-- @param str [string] The data to be decompressed.
-- @return [string/nil] If the decompression succeeds, return the decompressed
-- data. If the decompression fails, return nil. You should check if this return
-- value is non-nil to know if the decompression succeeds.
-- @return [integer] If the decompression succeeds, return the number of
-- unprocessed bytes in the input compressed data. This return value is a
-- positive integer if the input data is a valid compressed data appended by an
-- arbitary non-empty string. This return value is 0 if the input data does not
-- contain any extra bytes.<br>
-- If the decompression fails (The first return value of this function is nil),
-- this return value is undefined.
-- @see LibDeflate:CompressDeflate
function LibDeflate:DecompressDeflate(str)
  local arg_valid, arg_err = IsValidArguments(str)
  if not arg_valid then
    error(("Usage: LibDeflate:DecompressDeflate(str): " .. arg_err), 2)
  end
  return DecompressDeflateInternal(str)
end

return LibDeflate
