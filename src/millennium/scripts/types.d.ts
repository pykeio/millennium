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

export {};

declare global {
	type MillenniumCallbackId = number;

	type MillenniumPattern = 'brownfield' | 'isolation' | 'error';

	interface MillenniumWindow {
		label: string;
	}

	interface MillenniumInvokeArgs {
		__millenniumModule: string;
		message?: {
			cmd: string;
			[key: string]: any;
		};
	}

	interface MillenniumIsolationPayload {
		callback: string;
		error: string;
		data: any;
	}

	interface MillenniumIpcMessage {
		callback: MillenniumCallbackId;
		error: MillenniumCallbackId;
		[key: string]: any;
	}

	namespace __MILLENNIUM__ {
		function transformCallback<T>(callback: (res: T) => void | PromiseLike<void>, once?: boolean): MillenniumCallbackId;
	}

	function listen<T>(event: string, callback: (event: { payload: T, [k: string]: any }) => void | PromiseLike<void>): void;

	interface Window {
		__MILLENNIUM__: typeof __MILLENNIUM__;

		__MILLENNIUM_INVOKE__:
			(<T extends MillenniumCallbackId & { cmd: string }, R = any>(args: T) => Promise<R>) &
			(<T extends MillenniumInvokeArgs, R = any>(cmd: string, args: T) => Promise<R>);

		__MILLENNIUM_POST_MESSAGE__: (args: MillenniumIpcMessage) => void;
		__MILLENNIUM_IPC__: (args: MillenniumIpcMessage) => void;

		__MILLENNIUM_METADATA__: {
			__windows: Partial<MillenniumWindow>[];
		};

		__MILLENNIUM_PATTERN__: {
			pattern: MillenniumPattern;
		};
	}
};
