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
 * Exits immediately with the given exit code.
 *
 * This returns a Promise (because of IPC) and should be awaited. If you don't await
 * this, your code may continue running before the process actually exits, which might not
 * be desirable.
 */
export async function exit(exitCode: number = 0): Promise<void> {
	await invokeMillenniumCommand({
		__millenniumModule: 'Process',
		message: {
			cmd: 'exit',
			exitCode
		}
	});
}

/**
 * Relaunches the application.
 *
 * This returns a Promise (because of IPC) and should be awaited. If you don't await
 * this, your code may continue running before the process actually exits, which might not
 * be desirable.
 */
export async function relaunch(): Promise<void> {
	await invokeMillenniumCommand({
		__millenniumModule: 'Process',
		message: {
			cmd: 'relaunch'
		}
	});
}
