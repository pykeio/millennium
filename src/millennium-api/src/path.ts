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
 * Provides utilities for working with file and directory paths.
 *
 * @module
 */

import { BaseDirectory } from './fs';
import { invokeMillenniumCommand } from './helpers/millennium';
import { isWindows } from './platform';

const resolveBaseDirectory = async (directory: BaseDirectory) =>
	await invokeMillenniumCommand<string>({
		__millenniumModule: 'Path',
		message: {
			cmd: 'resolvePath',
			path: '',
			directory
		}
	});

/**
 * Returns the path to the suggested directory for your app config files.
 * Resolves to `{configDir}/{bundleIdentifier}`, where `bundleIdentifier` is configured in
 * `.millenniumrc > millennium > bundle > identifier`.
 */
export async function appDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.APP) }

/**
 * Returns the path to the user's audio directory.
 * - **Linux**: resolves to `XDG_MUSIC_DIR` ([`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/))
 * - **macOS**: resolves to `$HOME/Music`
 * - **Windows**: resolves to `{FOLDERID_Music}` (`C:\Users\{username}\Music`)
 */
export async function audioDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.AUDIO) }

/**
 * Returns the path to the recommended user cache directory.
 * - **Linux**: resolves to `XDG_CACHE_HOME` or `$HOME/.cache`
 * - **macOS**: resolves to `$HOME/Library/Caches`
 * - **Windows**: resolves to `{FOLDERID_LocalAppData}` (`C:\Users\{username}\AppData\Local`)
 */
export async function cacheDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.CACHE) }

/**
 * Returns the path to the user's config directory.
 * - **Linux**: resolves to `XDG_CONFIG_HOME` or `$HOME/.config`
 * - **macOS**: resolves to `$HOME/Library/Application Support`
 * - **Windows**: resolves to `{FOLDERID_LocalAppData}` (`C:\Users\{username}\AppData\Local`)
 */
export async function configDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.CONFIG) }

/**
 * Returns the path to the user's data directory.
 * - **Linux**: resolves to `XDG_DATA_HOME` or `$HOME/.local/share`
 * - **macOS**: resolves to `$HOME/Library/Application Support`
 * - **Windows**: resolves to `{FOLDERID_RoamingAppData}` (`C:\Users\{username}\AppData\Roaming`)
 */
export async function dataDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.DATA) }

/**
 * Returns the path to the user's desktop directory.
 * - **Linux**: resolves to `XDG_DESKTOP_DIR` ([`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/))
 * - **macOS**: resolves to `$HOME/Library/Desktop`
 * - **Windows**: resolves to `{FOLDERID_Desktop}` (`C:\Users\{username}\Desktop`)
 */
export async function desktopDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.DESKTOP) }

/**
 * Returns the path to the user's documents directory.
 * - **Linux**: resolves to `XDG_DOCUMENTS_DIR` ([`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/))
 * - **macOS**: resolves to `$HOME/Documents`
 * - **Windows**: resolves to `{FOLDERID_Documents}` (`C:\Users\{username}\Documents`)
 */
export async function documentsDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.DOCUMENT) }

/**
 * Returns the path to the user's downloads directory.
 * - **Linux**: resolves to `XDG_DOWNLOAD_DIR` ([`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/))
 * - **macOS**: resolves to `$HOME/Downloads`
 * - **Windows**: resolves to `{FOLDERID_Downloads}` (`C:\Users\{username}\Downloads`)
 */
export async function downloadsDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.DOWNLOAD) }

/**
 * Returns the path to the user's executable directory.
 * - **Linux**: resolves to `$XDG_BIN_HOME/../bin`, `$XDG_DATA_HOME/../bin`, or `$HOME/.local/bin`
 * - **macOS**: Not supported.
 * - **Windows**: Not supported.
 */
export async function executableDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.EXECUTABLE) }

/**
 * Returns the path to the user's font directory.
 * - **Linux**: resolves to `XDG_DATA_HOME/fonts` or `$HOME/.local/share/fonts`
 * - **macOS**: resolves to `$HOME/Library/Fonts`
 * - **Windows**: Not supported.
 */
export async function fontDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.FONT) }

/**
 * Returns the path to the user's home directory.
 * - **Linux**: resolves to `$HOME`
 * - **macOS**: resolves to `$HOME`
 * - **Windows**: resolves to `{FOLDERID_Profile}` (`C:\Users\{username}`)
 */
export async function homeDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.HOME) }

/**
 * Returns the path to the user's local data directory.
 * - **Linux**: resolves to `XDG_DATA_HOME` or `$HOME/.local/share`
 * - **macOS**: resolves to `$HOME/Library/Application Support`
 * - **Windows**: resolves to `{FOLDERID_LocalAppData}` (`C:\Users\{username}\AppData\Local`)
 */
export async function localDataDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.LOCALDATA) }

/**
 * Returns the path to the user's picture directory.
 * - **Linux**: resolves to `XDG_PICTURES_DIR` ([`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/))
 * - **macOS**: resolves to `$HOME/Pictures`
 * - **Windows**: resolves to `{FOLDERID_Pictures}` (`C:\Users\{username}\Pictures`)
 */
export async function pictureDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.PICTURES) }

/**
 * Returns the path to the user's public directory.
 * - **Linux**: resolves to `XDG_PUBLICSHARE_DIR` ([`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/))
 * - **macOS**: resolves to `$HOME/Public`
 * - **Windows**: resolves to `{FOLDERID_Public}` (`C:\Users\{username}\Public`)
 */
export async function publicDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.PUBLIC) }

/**
 * Returns the path to the user's resource directory.
 */
export async function resourceDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.RESOURCE) }

/**
 * Returns the path to the user's runtime directory.
 * - **Linux**: resolves to `XDG_RUNTIME_DIR` ([`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/))
 * - **macOS**: Not supported.
 * - **Windows**: Not supported.
 */
export async function runtimeDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.RUNTIME) }

/**
 * Returns the path to the user's template directory.
 * - **Linux**: resolves to `XDG_TEMPLATES_DIR` ([`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/))
 * - **macOS**: Not supported.
 * - **Windows**: resolves to `{FOLDERID_Templates}` (`C:\Users\{username}\Templates`)
 */
export async function templateDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.TEMPLATES) }

/**
 * Returns the path to the user's videos directory.
 * - **Linux**: resolves to `XDG_VIDEOS_DIR` ([`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/))
 * - **macOS**: resolves to `$HOME/Movies`
 * - **Windows**: resolves to `{FOLDERID_Videos}` (`C:\Users\{username}\Videos`)
 */
export async function videosDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.VIDEOS) }

/**
 * Returns the path to the suggested log directory.
 * - **Linux**: resolves to `{configDir}/{bundleIdentifier}`
 * - **macOS**: resolves to `{homeDir}/Library/Logs/{bundleIdentifier}`
 * - **Windows**: resolves to `{configDir}/{bundleIdentifier}`
 */
export async function logDir(): Promise<string> { return resolveBaseDirectory(BaseDirectory.LOG) }

/** Platform-specific path segment separator. */
export const sep = isWindows() ? '\\' : '/';

/** Platform-specific path segment delimiter. */
export const delimiter = isWindows() ? ';' : ':';

/**
 * Resolves a sequence of paths or path segments intoan absolute path.
 *
 * @param paths A sequence of paths or path segments.
 */
export async function resolve(...paths: string[]): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Path',
		message: {
			cmd: 'resolve',
			paths
		}
	});
}

/**
 * Normalizes the given `path`, resolving `'..'` and `'.'` segments and resolving symbolic links.
 */
export async function normalize(path: string): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Path',
		message: {
			cmd: 'normalize',
			path
		}
	});
}

/**
 * Joins all given `path` segments together using the platform-specific separator as a delimiter, then normalizes the resulting path.
 *
 * @param paths A sequence of path segments.
 */
export async function join(...paths: string[]): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Path',
		message: {
			cmd: 'join',
			paths
		}
	});
}

/**
 * Returns the directory name of a given `path`. Trailing directory separators are ignored.
 */
export async function dirname(path: string): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Path',
		message: {
			cmd: 'dirname',
			path
		}
	});
}

/**
 * Returns the extension of the given `path`, including the `'.'`.
 * If there is no extension, an empty string is returned.
 */
export async function extname(path: string): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Path',
		message: {
			cmd: 'extname',
			path
		}
	});
}

/**
 * Returns the last portion of a `path` (usually the filename). Trailing directory separators are ignored.
 *
 * @param ext An optional file extension to be removed from the returned path.
 */
export async function basename(path: string, ext?: string): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Path',
		message: {
			cmd: 'basename',
			path,
			ext
		}
	});
}

export async function isAbsolute(path: string): Promise<boolean> {
	return invokeMillenniumCommand<boolean>({
		__millenniumModule: 'Path',
		message: {
			cmd: 'isAbsolute',
			path
		}
	});
}
