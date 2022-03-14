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
 * Provides operating system-related utility functions and properties.
 */

import { LiteralUnion } from 'type-fest';

import { invokeMillenniumCommand } from './helpers/millennium';
import { isWindows } from './platform';

/** The line ending character(s) used by the OS. */
export const eol = isWindows() ? '\r\n' : '\n';

/**
 * Returns a string identifying the operating system.
 *
 * **NOTE**: `darwin` is macOS, `win32` is Windows. iOS is separate from `darwin`.
 */
export async function platform(): Promise<LiteralUnion<
	| 'linux'
	| 'darwin'
	| 'ios'
	| 'freebsd'
	| 'dragonfly'
	| 'netbsd'
	| 'openbsd'
	| 'solaris'
	| 'android'
	| 'win32',
	string
>> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Os',
		message: {
			cmd: 'platform'
		}
	});
}

/**
 * Returns a string identifying the kernel version.
 */
export async function version(): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Os',
		message: {
			cmd: 'version'
		}
	});
}

/**
 * Returns the type of the operating system.
 */
export async function type(): Promise<LiteralUnion<'Linux' | 'Darwin' | 'Windows_NT', string>> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Os',
		message: {
			cmd: 'type'
		}
	});
}

/**
 * Returns the system architecture for which this app was compiled for.
 */
export async function arch(): Promise<LiteralUnion<
	| 'x86'
	| 'x86_64'
	| 'arm'
	| 'aarch64'
	| 'mips'
	| 'mips64'
	| 'powerpc'
	| 'powerpc64'
	| 'riscv64'
	| 's390x'
	| 'sparc64',
	string
>> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Os',
		message: {
			cmd: 'arch'
		}
	});
}

/**
 * Returns the operating system's default directory for temporary files.
 */
export async function tmpdir(): Promise<string> {
	return invokeMillenniumCommand<string>({
		__millenniumModule: 'Os',
		message: {
			cmd: 'tempdir'
		}
	});
}
