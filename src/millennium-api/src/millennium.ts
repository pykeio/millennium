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

import { isWindows } from './platform';

/**
 * @module
 * Invoke custom commands.
 *
 * This package is also accessible with `window.Millennium.millennium` when `.millenniumrc > build > withGlobalMillennium` is enabled.
 */

declare global {
	interface Window {
		__MILLENNIUM_IPC__: (message: any) => void;
		ipc: {
			postMessage: (args: string) => void;
		}
	}
}

function uid(): number {
	return window.crypto.getRandomValues(new Uint32Array(1))[0];
}

/**
 * Transform a callback function to a string identifier that can be passed to the backend.
 * The backend uses the identifier to `eval()` the callback.
 *
 * @returns A unique identifier associated with the callback function.
 */
export function transformCallback(callback?: (response: any) => void, once = false): number {
	const identifier = uid();
	const prop = `_${identifier}`;
	Object.defineProperty(window, prop, {
		value: (result: any) => {
			if (once)
				delete window[prop as any];

			return callback?.(result);
		},
		writable: true,
		enumerable: false,
		configurable: true
	});
	return identifier;
}

interface InvokeArgs {
	[key: string]: unknown;
}

/**
 * Sends a message to the backend.
 *
 * @param cmd The command name.
 * @param args The optional arguments to pass to the command.
 * @returns A Promise resolving or rejecting with the result of the command.
 */
export async function invoke<T>(cmd: string, args: InvokeArgs = {}): Promise<T> {
	return new Promise((resolve, reject) => {
		const callback = transformCallback((e: T) => {
			resolve(e);
			delete window[`_${error}` as any];
		}, true);
		const error = transformCallback((e: string) => {
			reject(new Error(e));
			delete window[`_${callback}` as any];
		}, true);

		window.__MILLENNIUM_IPC__({
			cmd,
			callback,
			error,
			...args
		});
	});
}

/**
 * Convert a device file path to an URL that can be loaded by the webview.
 * Note that `asset:` and `https://asset.localhost` must be allowed on the `csp` value configured on `.millenniumrc > millennium > security`.
 * Example CSP value: `"csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost"` to use the asset protocol on image sources.
 *
 * Additionally, the `asset` must be allowlisted under `.millenniumrc > millennium > allowlist > protocol`,
 * and its access scope must be defined on the `assetScope` array on the same `protocol` object.
 *
 * @param filePath The file path.
 * @param protocol The protocol to use. Defaults to `asset`. You only need to set this when using a custom protocol.
 * @example
 * ```typescript
 * import { appDir, join } from '@pyke/millennium-api/path'
 * import { convertFileSrc } from '@pyke/millennium-api/millennium'
 * const appDirPath = await appDir()
 * const filePath = await join(appDir, 'assets/video.mp4')
 * const assetUrl = convertFileSrc(filePath)
 *
 * const video = document.getElementById('my-video')
 * const source = document.createElement('source')
 * source.type = 'video/mp4'
 * source.src = assetUrl
 * video.appendChild(source)
 * video.load()
 * ```
 *
 * @return the URL that can be used as source on the webview.
 */
export function convertFileSrc(filePath: string, protocol = 'asset'): string {
	return isWindows()
		? `https://${protocol}.localhost/${filePath}`
		: `${protocol}://${filePath}`;
}
