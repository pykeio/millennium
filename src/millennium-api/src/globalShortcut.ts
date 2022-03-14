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
import { transformCallback } from './millennium';

export type ShortcutHandler = (shortcut: string) => void;

/**
 * Register a global shortcut.
 *
 * @param shortcut Shortcut definition, modifiers and key separated by "+", e.g. `CmdOrControl+Shift+Q`
 * @param handler Shortcut handler callback, takes the triggered shortcut as an argument.
 */
export async function register(shortcut: string, handler: ShortcutHandler): Promise<void> {
	return await invokeMillenniumCommand({
		__millenniumModule: 'GlobalShortcut',
		message: {
			cmd: 'register',
			shortcut,
			handler: transformCallback(handler)
		}
	});
}

/**
 * Register a collection of global shortcuts.
 *
 * @param shortcuts Shortcut definitions, modifiers and key separated by "+", e.g. `CmdOrControl+Shift+Q`
 * @param handler Shortcut handler callback, takes the triggered shortcut as an argument.
 */
export async function registerAll(shortcuts: string[], handler: ShortcutHandler): Promise<void> {
	return await invokeMillenniumCommand({
		__millenniumModule: 'GlobalShortcut',
		message: {
			cmd: 'registerAll',
			shortcuts,
			handler: transformCallback(handler)
		}
	});
}

/**
 * Determines whether the given shortcut is registered by this application or not.
 *
 * @param shortcut Shortcut definition, modifiers and key separated by "+", e.g. `CmdOrControl+Shift+Q`
 */
export async function isRegistered(shortcut: string): Promise<boolean> {
	return await invokeMillenniumCommand({
		__millenniumModule: 'GlobalShortcut',
		message: {
			cmd: 'isRegistered',
			shortcut
		}
	});
}

/**
 * Unregisters a global shortcut.
 *
 * @param shortcut Shortcut definition, modifiers and key separated by "+", e.g. `CmdOrControl+Shift+Q`
 */
export async function unregister(shortcut: string): Promise<void> {
	return await invokeMillenniumCommand({
		__millenniumModule: 'GlobalShortcut',
		message: {
			cmd: 'unregister',
			shortcut
		}
	});
}

/**
 * Unregisters all global shortcuts registered by the application.
 */
export async function unregisterAll(): Promise<void> {
	return await invokeMillenniumCommand({
		__millenniumModule: 'GlobalShortcut',
		message: {
			cmd: 'unregisterAll'
		}
	});
}
