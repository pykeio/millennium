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

import { invoke } from '../millennium';

export type MillenniumModule =
	| 'App'
	| 'Fs'
	| 'Path'
	| 'Os'
	| 'Window'
	| 'Shell'
	| 'Event'
	| 'Internal'
	| 'Dialog'
	| 'Cli'
	| 'Notification'
	| 'Http'
	| 'GlobalShortcut'
	| 'Process'
	| 'Clipboard';

export interface MillenniumCommand {
	__millenniumModule: MillenniumModule;
	[key: string]: unknown;
}

export async function invokeMillenniumCommand<T>(command: MillenniumCommand): Promise<T> {
	return invoke<T>('millennium', command);
}
