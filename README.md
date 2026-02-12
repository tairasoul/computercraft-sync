# computercraft tweaked sync server
or technically any computer mod that runs lua, if you modify sync.lua a little bit

made because no other sync servers existed

## note

unwrap() is used in a lot of places in the project as of current, and it should not be treated as a production-ready program (wherever you'd call "production" for computercraft)

ohkami also seems to be a little weird with the websockets and waiting for them to close, you have to kill the process manually instead of ctrl+C'ing it else it'll hang until timeout is reached (which is set to 4 hours)

minify also currently does not use darklua's Dense generator as it removes comments and comments have to be kept in order to keep goto's without the parser erroring

## usage

in order to use this, download the binary from Releases, put it in a project folder (`project.ron` should be at the root), and run it

then, in cc, run `wget run http://url-to-server:port/download` to download everything needed (port is optional if it's just a DNS record)

ngrok tcp tunnels can be used for this

after that, you can run `sync` to print sync.lua's usage, run `sync url-to-server:port` to list channels, or run `sync url-to-server:port channels to sync` in order to subscribe to a list of space-separated channels

for example, to subscribe to channels `common` and `ui`, run `sync url-to-server:port common ui`

## config

configuring a project is done in a file called `project.ron`

this file looks like this

```ron
Project(
	root_dir: "src", // relative to project.ron
	max_uncompressed_request_size: 100000, // how many bytes can a request be before it needs to be chunked when sending it
	port: 10234, // port to run the server on
	minify: true, // default to minifying files
	deflate_trickery: true, // default to doing deflate bullshit on files
	require_prefix: "/", // what to prefix requires with by default
	prefix_exclusions: ["some.library"], // requires to exclude from prefixing
	lz_on_deflate: true, // should lz4 be used to compress libdeflate.lua
	sync_interval: 2, // how long to wait between checks for syncing (in seconds)
	items: [
		// channels in the project
		ProjectItem(
			type: Library, // can be Resource, Library or Script
			channel_name: "example-library", // name of the channel, used when subscribing
			// minify, deflate_trickery, require_prefix and prefix_exclusions work here too
			// whatever you set these to takes priority over project root
			required_channels: ["example-dependency"], // optional, channels to implicitly subscribe to and send to the client alongside this one
			directories: [ // optional
				// directories to sync
				Directory(
					path: "example-dir", // relative to project root, so "src/example-dir"
					// minify, deflate_trickery, require_prefix and prefix_exclusions work here too
					// whatever you set these to takes priority over project root and the channel
				)
			],
			files: [ // optional
				// specific files to sync
				File(
					path: "example-file.lua", // relative to project root
					cc_path: "startup.lua", // optional, where to place the file on the computer
					// minify, deflate_trickery, require_prefix and prefix_exclusions work here too
					// whatever you set these to takes priority over project root, the channel and the directory (if this file is in one)
					bundle: true // bundle this file.
					// when bundling, you should not add the channels that this would otherwise require,
					// as that will just sync the entire channel alongside the bundled file
				)
			]
		)
	]
)
```

## channel types

channel types have few differences, those that do are listed here

### Resource

should be used for pure data files, if deflate_trickery'd will be turned into a valid lua file you can require to get the original data

minify & bundle do nothing on these channels

### Library & Script

used for actual lua files, both act exactly the same

## defaults

if not specified, overridden by the corresponding setting on the directory, channel or project:

minify: false

deflate_trickery: false

require_prefix: none

prefix_exclusions: none

### Project

lz_on_deflate: false

sync_interval: 1

### File

cc_path: same as path

## programming crimes

deflate_trickery is the primary horrid crime in this project

when using deflate_trickery, the file is compressed with the highest compression possible using `flate2`, then encoded into base85 and embedded into a file

the resulting file looks something like this

`return load(require("/cc-sync/libdeflate").libDeflate:DecompressDeflate(select(2, require("/cc-sync/base8\").decode("base85"))))(...)` where base85 is the encoded libdeflate data

load()(...) is excluded if it's a resource

## server routes

/ - get a libdeflated newline-separated list of channels formatted as `channel_name - channel_type`

/libdeflate.lua - get a minified (and lz4'd, if the option is set) version of lua/libdeflate.lua

/sync.lua - get a minified version of lua/sync.lua

/base85.lua - get a minified version of lua/base85.lua

/lz4.lua - get a minified version of lua/llz4.lua

/base-sync.lua - get the unminified (still bundled with msgpack) version of lua/sync.lua

/base-libdeflate.lua - get the unminified version of lua/libdeflate.lua

/base-base85.lua - get the unminified version of lua/base85.lua

/base-lz4.lua - get the unminified version of lua/llz4.lua

/download - get the script for downloading everything necessary for sync.lua to run, from the regular routes

/download-nomin - get the script for downloading everything necessary for sync.lua to run, from the base* routes

/subscribe?channels=comma,separated,list - subscribe to channels, channels are separated by commas in the channels parameter

## potential improvements

base85 increases the size of the data it's representing almost as much as base45, but it should be possible to make a base94 variant while attempting to keep base45's storage optimizations

# attributions

[`lua/base85.lua`](https://github.com/Anaminus/roblox-library/blob/master/modules/Base85/init.lua) is licensed under the MIT-0 license, original license follows

<details><summary>lua/base85.lua LICENSE (click to view)</summary>
MIT No Attribution

Copyright 2022 Anaminus

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
</details>

[`lua/libdeflate.lua`](https://github.com/SafeteeWoW/LibDeflate/blob/main/LibDeflate.lua) is licensed under the Zlib license

lua/libdeflate.lua is altered to remove WoW related functions, cli, internals and to allow for use in TypescriptToLua

original license follows


<details><summary>lua/libdeflate.lua LICENSE (click to view)</summary>
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
</details>

[`lua/llz4.lua`](https://github.com/RiskoZoSlovenska/llz4/blob/main/llz4.lua) is licensed under the MIT license

altered to remove the code that allows for lua runtime compat since the target is cc

goto is also removed because darklua doesnt support those

original license follows

<details><summary>lua/llz4.lua LICENSE (click to view)</summary>
MIT License

Copyright (c) 2025 RiskoZoSlovenska

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
</details>

[`lua/msgpack.lua`](https://github.com/kieselsteini/msgpack/blob/master/msgpack.lua) is licensed under the Unlicense, original license follows

<details><summary>lua/msgpack LICENSE (click to view)</summary>
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org/>
</details>