/**
 * MIT License
 *
 * Copyright (c) 2022 pyke.io
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is furnished to do
 * so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

// @ts-nocheck module/target error; this is meant to be run directly, not transpiled

import { build } from 'esbuild';
import { constants } from 'fs';
import { access } from 'fs/promises';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

// Chromium 79 is the earliest version of Edge WebView2.
// Minimum macOS version is 10.12, which ships with Safari 12.1, but on Linux the oldest version of libwebkit2gtk-4.0-37 I could find on Debian was in Stretch,
// which came out shortly before Safari 11. There's probably a large chance that Linux systems will have Safari 11+, if not a much later version by now, almost
// 5 years after the release of Stretch. Also, the minimum Safari version esbuild can (currently) transpile to is Safari 11 anyways...
const target = [ 'chrome79', 'safari11' ];

await build({
	bundle: true,
	minify: true,
	format: 'esm',
	platform: 'browser',
	entryPoints: [ 'src/index.ts' ],
	target,
	outfile: 'dist/millennium-api.js'
});

async function exists(path) {
	try {
		await access(path, constants.F_OK);
		return true;
	} catch {
		return false;
	}
}

const __dirname = dirname(fileURLToPath(import.meta.url));
if (!(await exists(resolve(__dirname, '../millennium')) && await exists(resolve(__dirname, '../../Cargo.toml')))) return;

await build({
	bundle: true,
	minify: true,
	format: 'iife',
	platform: 'browser',
	entryPoints: [ 'src/index.ts' ],
	globalName: 'Millennium',
	footer: { js: ';Object.defineProperty(window,"Millennium",{value:Millennium,writable:false,configurable:false,enumerable:true});function _DF(e){const d=Object.getOwnPropertyNames(e);for(const g of d)if(typeof e[g]=="object")_DF(e[g]);Object.freeze(e)}_DF(window.Millennium);' },
	target,
	outfile: resolve(__dirname, '../millennium/scripts/bundle.js')
});
