/**
 * Copyright 2022 pyke.io
 *           2019-2021 Tauri Programme within The Commons Conservancy
 *                     [https://tauri.studio/]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * Access the file system.
 *
 * This package is also accessible with `window.Millennium.fs` when `.millenniumrc > build > withGlobalMillennium` is set to true.
 *
 * The APIs must be allowlisted on `.millenniumrc`:
 * ```json
 * {
 * 	"millennium": {
 * 		"allowlist": {
 * 			"fs": {
 * 				"all": true, // enable all FS APIs
 * 				"readFile": true,
 * 				"writeFile": true,
 * 				"readDir": true,
 * 				"copyFile": true,
 * 				"createDir": true,
 * 				"removeDir": true,
 * 				"removeFile": true,
 * 				"renameFile": true
 * 			}
 * 		}
 * 	}
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 *
 * ## Security
 *
 * This module prevents path traversal, not allowing absolute paths or parent dir components
 * (i.e. "/usr/path/to/file" or "../path/to/file" paths are not allowed).
 * Paths accessed with this API must be relative to one of the [[BaseDirectory | base directories]]
 * so if you need access to arbitrary filesystem paths, you must write such logic on the core layer instead.
 *
 * The API has a scope configuration that forces you to restrict the paths that can be accessed using glob patterns.
 *
 * The scope configuration is an array of glob patterns describing folder paths that are allowed.
 * For instance, this scope configuration only allows accessing files on the
 * *databases* folder of the [[path.appDir | $APP directory]]:
 * ```json
 * {
 * 	"millennium": {
 * 		"allowlist": {
 * 			"fs": {
 * 				"scope": [ "$APP/databases/*" ]
 * 			}
 * 		}
 * 	}
 * }
 * ```
 *
 * Notice the use of the `$APP` variable. The value is injected at runtime, resolving to the [[path.appDir | app directory]].
 * The available variables are:
 * [[path.audioDir | `$AUDIO`]], [[path.cacheDir | `$CACHE`]], [[path.configDir | `$CONFIG`]], [[path.dataDir | `$DATA`]],
 * [[path.localDataDir | `$LOCALDATA`]], [[path.desktopDir | `$DESKTOP`]], [[path.documentDir | `$DOCUMENT`]],
 * [[path.downloadDir | `$DOWNLOAD`]], [[path.executableDir | `$EXE`]], [[path.fontDir | `$FONT`]], [[path.homeDir | `$HOME`]],
 * [[path.pictureDir | `$PICTURE`]], [[path.publicDir | `$PUBLIC`]], [[path.runtimeDir | `$RUNTIME`]],
 * [[path.templateDir | `$TEMPLATE`]], [[path.videoDir | `$VIDEO`]], [[path.resourceDir | `$RESOURCE`]], [[path.appDir | `$APP`]].
 *
 * Trying to execute any API with a URL not configured on the scope results in a promise rejection due to denied access.
 *
 * Note that this scope applies to **all** APIs on this module.
 *
 * @module
 */

import { invokeMillenniumCommand } from './helpers/millennium';

export enum BaseDirectory {
	AUDIO = 1,
	CACHE,
	CONFIG,
	DATA,
	LOCALDATA,
	DESKTOP,
	DOCUMENT,
	DOWNLOAD,
	EXECUTABLE,
	FONT,
	HOME,
	PICTURES,
	PUBLIC,
	RUNTIME,
	TEMPLATES,
	VIDEOS,
	RESOURCE,
	APP,
	LOG
}

interface FsOptions {
	dir?: BaseDirectory;
}

interface FsDirOptions {
	dir?: BaseDirectory;
	recursive?: boolean;
}

/** Options object used when writing a UTF-8 string to a file. */
interface FsTextFileOptions {
	path: string;
	contents: string;
}

/** Options object used when writing binary data to a file. */
interface FsBinaryFileOptions {
	path: string;
	contents: Iterable<number> | ArrayLike<number>;
}

interface FileEntry {
	path: string;
	/** Name of the directory/file. Can be null if the path terminates with `..`. */
	name?: string;
	/** Children of this entry if it's a directory, null otherwise. */
	children?: FileEntry[];
}

/**
 * Reads a file as a UTF-8 encoded string.
 */
export async function readTextFile(filePath: string, options: FsOptions = {}): Promise<string> {
	return await invokeMillenniumCommand<string>({
		__millenniumModule: 'Fs',
		message: {
			cmd: 'readTextFile',
			path: filePath,
			options
		}
	});
}

/**
 * Reads a file as a raw byte array. In most cases, you'll probably want `readTextFile`.
 */
export async function readBinaryFile(filePath: string, options: FsOptions = {}): Promise<Uint8Array> {
	return await invokeMillenniumCommand<Uint8Array>({
		__millenniumModule: 'Fs',
		message: {
			cmd: 'readFile',
			path: filePath,
			options
		}
	});
}

/**
 * Reads a file as a UTF-8 encoded string.
 */
export async function readFile(path: string, encoding: 'utf8' | 'utf-8'): Promise<string>;
/**
 * Reads a file as a UTF-8 encoded string.
 *
 * @param options Additional file reading options.
 */
export async function readFile(path: string, encoding: 'utf8' | 'utf-8', options: FsOptions): Promise<string>;
/**
 * Reads a file as a raw binary array. In most cases, you'll probably want to read as UTF-8.
 * Currently, UTF-8 is the only other supported encoding. To read other encodings, you'll have to read a file as
 * `binary` and then decode it yourself using `TextDecoder`.
 */
export async function readFile(path: string, encoding: 'binary'): Promise<Uint8Array>;
/**
 * Reads a file as a raw binary array. In most cases, you'll probably want to read as UTF-8.
 * Currently, UTF-8 is the only other supported encoding. To read other encodings, you'll have to read a file as
 * `binary` and then decode it yourself using `TextDecoder`.
 *
 * @param options Additional file reading options.
 */
export async function readFile(path: string, encoding: 'binary', options: FsOptions): Promise<Uint8Array>;
/**
 * Reads a file as a raw binary array. In most cases, you'll probably want to read as UTF-8.
 * Currently, UTF-8 is the only other supported encoding. To read other encodings, you'll have to read a file as
 * `binary` and then decode it yourself using `TextDecoder`.
 *
 * @param options Additional file reading options.
 */
export async function readFile(path: string, options: FsOptions): Promise<Uint8Array>;
export async function readFile(path: string, encoding: FsOptions | 'utf8' | 'utf-8' | 'binary', options: FsOptions = {}): Promise<string | Uint8Array> {
	if (typeof encoding !== 'string') {
		options = encoding;
		encoding = 'binary';
	}

	switch (encoding) {
		case 'utf8':
		case 'utf-8':
			return await readTextFile(path, options);
		case 'binary':
			return await readBinaryFile(path, options);
		default:
			throw new Error(`Unsupported encoding: '${encoding}'. Consider reading as binary and using \`TextDecoder\`.`);
	}
}

/**
 * Writes a UTF-8 string to a file.
 */
export async function writeTextFile(file: FsTextFileOptions, options: FsOptions = {}): Promise<void> {
	return await invokeMillenniumCommand<void>({
		__millenniumModule: 'Fs',
		message: {
			cmd: 'writeTextFile',
			path: file.path,
			contents: Array.from(new TextEncoder().encode(file.contents)),
			options
		}
	});
}

/**
 * Writes a raw byte array to a file.
 */
export async function writeBinaryFile(file: FsBinaryFileOptions, options: FsOptions = {}): Promise<void> {
	return await invokeMillenniumCommand<void>({
		__millenniumModule: 'Fs',
		message: {
			cmd: 'writeFile',
			path: file.path,
			contents: Array.from(file.contents),
			options
		}
	});
}

type TypedArray =
	| Int8Array
	| Uint8Array
	| Uint8ClampedArray
	| Int16Array
	| Uint16Array
	| Int32Array
	| Uint32Array
	| Float32Array
	| Float64Array
	| BigInt64Array
	| BigUint64Array;

type ArrayBufferLike = ArrayBuffer | SharedArrayBuffer;

type BufferContainer =
	| TypedArray
	| DataView;

export async function writeFile(path: string, contents: string, options: FsOptions): Promise<void>;
export async function writeFile(path: string, contents: TypedArray | BufferContainer, options: FsOptions): Promise<void>;
export async function writeFile(path: string, contents: ArrayBufferLike, options: FsOptions): Promise<void>;
export async function writeFile(path: string, contents: string | TypedArray | BufferContainer | ArrayBufferLike, options: FsOptions): Promise<void> {
	if (typeof contents === 'string')
		return await writeTextFile({ path, contents }, options);
	else if (contents instanceof ArrayBuffer || contents instanceof SharedArrayBuffer)
		return await writeBinaryFile({ path, contents: Array.from(new Uint8Array(contents as ArrayBufferLike)) }, options);
	else if (
		[
			DataView,
			Int8Array, Uint8Array, Uint8ClampedArray,
			Int16Array, Uint16Array,
			Int32Array, Uint32Array, Float32Array
		].some(t => contents instanceof t)
		|| (typeof Float64Array !== 'undefined' && contents instanceof Float64Array)
		|| (typeof BigInt64Array !== 'undefined' && contents instanceof BigInt64Array)
		|| (typeof BigUint64Array !== 'undefined' && contents instanceof BigUint64Array)
	)
		return await writeBinaryFile({ path, contents: Array.from(new Uint8Array(contents.buffer.slice(contents.byteOffset, contents.byteOffset + contents.byteLength))) }, options);
	else
		throw new Error(`Unsupported contents type: '${{}.toString.call(contents)}'.`);
}

const invokeBase = <T>(cmd: string, options: Record<string, any>): Promise<T> =>
	invokeMillenniumCommand<T>({
		__millenniumModule: 'Fs',
		message: {
			cmd,
			...options
		}
	});

/**
 * Returns a list of entries in the directory, including files and other directories.
 */
export async function readdir(dir: string, options: FsDirOptions = {}): Promise<FileEntry[]> {
	return await invokeBase<FileEntry[]>('readDir', { path: dir, options });
}

/**
 * Creates a directory. If the `recursive` option is enabled, any of the path's missing parent components
 * will be created as well (similar to `mkdir -p` on Linux). If the `recursive` option is disabled, the
 * parent directories must exist, or an error will be thrown.
 */
export async function mkdir(dir: string, options: FsDirOptions = {}): Promise<void> {
	return await invokeBase<void>('createDir', { path: dir, options });
}

/**
 * Removes a directory. If the `recursive` option is not enabled and the directory is not empty, an error
 * will be thrown. If the `recursive` option is enabled, all of the directory's contents will be removed.
 * Be careful when using the `recursive` option!
 */
export async function rmdir(dir: string, options: FsDirOptions = {}): Promise<void> {
	return await invokeBase<void>('removeDir', { path: dir, options });
}

/**
 * Copies a file to a destination.
 */
export async function copyFile(source: string, destination: string, options: FsOptions = {}): Promise<void> {
	return await invokeBase<void>('copyFile', { source, destination, options });
}

/**
 * Removes a file permanently. This does not move the file to the recycle bin/trash on Windows/macOS.
 */
export async function removeFile(file: string, options: FsOptions = {}): Promise<void> {
	return await invokeBase<void>('removeFile', { path: file, options });
}

/**
 * Renames/moves a file or directory.
 */
export async function rename(oldPath: string, newPath: string, options: FsOptions = {}): Promise<void> {
	return await invokeBase<void>('rename', { oldPath, newPath, options });
}

/**
 * Checks if a file or directory exists.
 */
export async function exists(path: string, options: FsOptions = {}): Promise<boolean> {
	return await invokeBase<boolean>('exists', { path, options });
}
