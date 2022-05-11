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

import { invokeMillenniumCommand } from './helpers/millennium';

/**
 * Writes plain text to the clipboard.
 *
 * @returns A promise indicating the success or failure of the operation.
 */
export async function writeText(text: string): Promise<void> {
	return invokeMillenniumCommand({
		__millenniumModule: 'Clipboard',
		message: {
			cmd: 'writeText',
			data: text
		}
	});
}

/**
 * Gets the clipboard content as plain text.
 *
 * @returns A promise that resolves to the clipboard content as plain text.
 */
export async function readText(): Promise<string | null> {
	return invokeMillenniumCommand({
		__millenniumModule: 'Clipboard',
		message: {
			cmd: 'readText',
			// if data is not set, `serde` will ignore the custom deserializer that is set when the API is not allowlisted
			data: null
		}
	});
}
