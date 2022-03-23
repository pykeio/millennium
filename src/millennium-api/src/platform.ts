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

declare global {
	interface Navigator {
		userAgentData?: {
			platform: string;
			mobile: boolean;
			brands: { brand: string; version: string }[];
		};
	}
}

export function isBrowser(): boolean {
	return typeof window !== 'boolean';
}

export function isLinux(): boolean {
	return isBrowser()
		? /linux/i.test(navigator.userAgentData?.platform ?? navigator.userAgent)
		: process.platform === 'linux';
}

export function isWindows(): boolean {
	return isBrowser()
		? /win/i.test(navigator.userAgentData?.platform ?? navigator.userAgent)
		: process.platform === 'win32';
}

export function isMacOS(): boolean {
	return isBrowser()
		? /mac/i.test(navigator.userAgentData?.platform ?? navigator.userAgent)
		: process.platform === 'darwin';
}
