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
 * Access the system shell.
 * Allows you to spawn child processes and manage files and URLs using their default application.
 *
 * This package is also accessible with `window.Millennium.shell` when `.millenniumrc > build > withGlobalMillennium` is set to true.
 *
 * The APIs must be allowlisted on `.millenniumrc`:
 * ```json
 * {
 * 	"millennium": {
 * 		"allowlist": {
 * 			"shell": {
 * 				"all": true, // enable all shell APIs
 * 				"execute": true, // enable process spawn APIs
 * 				"sidecar": true, // enable spawning sidecars
 * 				"open": true // enable opening files/URLs using the default program
 * 			}
 * 		}
 * 	}
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 *
 * ## Security
 *
 * This API has a scope configuration that forces you to restrict the programs and arguments that can be used.
 *
 * ### Restricting access to the [[open | `open`]] API
 *
 * On the allowlist, `open: true` means that the [[open]] API can be used with any URL,
 * as the argument is validated with the `^https?://` regex.
 * You can change that regex by changing the boolean value to a string, e.g. `open: ^https://github.com/`.
 *
 * ### Restricting access to the [[Command | `Command`]] APIs
 *
 * The `shell` allowlist object has a `scope` field that defines an array of CLIs that can be used.
 * Each CLI is a configuration object `{ name: string, cmd: string, sidecar?: bool, args?: boolean | Arg[] }`.
 *
 * - `name`: the unique identifier of the command, passed to the [[Command.constructor | Command constructor]].
 * If it's a sidecar, this must be the value defined on `.millenniumrc > millennium > bundle > externalBin`.
 * - `cmd`: the program that is executed on this configuration. If it's a sidecar, this value is ignored.
 * - `sidecar`: whether the object configures a sidecar or a system program.
 * - `args`: the arguments that can be passed to the program. By default no arguments are allowed.
 * 	- `true` means that any argument list is allowed.
 * 	- `false` means that no arguments are allowed.
 * 	- otherwise an array can be configured. Each item is either a string representing the fixed argument value or a
 * 	  `{ validator: string }` that defines a regex validating the argument value.
 *
 * #### Example scope configuration
 *
 * CLI: `git commit -m "the commit message"`
 *
 * Configuration:
 * ```json
 * {
 * 	"scope": {
 * 		"name": "run-git-commit",
 * 		"cmd": "git",
 * 		"args": ["commit", "-m", { "validator": "\\S+" }]
 * 	}
 * }
 * ```
 * Usage:
 * ```typescript
 * import { Command } from '@pyke/millennium-api/shell'
 * new Command('run-git-commit', ['commit', '-m', 'the commit message'])
 * ```
 *
 * Trying to execute any API with a program not configured on the scope results in a promise rejection due to denied access.
 *
 * @module
 */

import { invokeMillenniumCommand } from './helpers/millennium';
import { isLinux, isMacOS, isWindows } from './platform';
import { transformCallback } from './millennium';

interface SpawnOptions {
	/** Current working directory. */
	cwd?: string;
	/** Environment variables. Set to `null` to clear all environment variables. */
	env?: { [key: string]: string };
}

interface InternalSpawnOptions extends SpawnOptions {
	sidecar?: boolean;
}

interface ChildProcess {
	/** Exit code of the process. `null` if the process was terminated by a signal on Unix. */
	code: number | null;
	/** If the process was terminated by a signal, this represents that signal. */
	signal: number | null;
	/** The data that the process wrote to `stdout`. */
	stdout: string;
	/** The data that the process wrote to `stderr`. */
	stderr: string;
}

/**
 * Spawns a process.
 *
 * @param onEvent Event handler.
 * @param program The name of the scoped command.
 * @param args Program arguments.
 * @param options Configuration for the spawned process.
 * @returns A Promise resolving the process's PID.
 */
async function execute(onEvent: (event: CommandEvent) => void, program: string, args: string | string[] = [], options?: InternalSpawnOptions): Promise<number> {
	if (typeof args === 'object')
		Object.freeze(args);

	return invokeMillenniumCommand<number>({
		__millenniumModule: 'Shell',
		message: {
			cmd: 'execute',
			program,
			args,
			options,
			onEventFn: transformCallback(onEvent)
		}
	});
}

class EventEmitter<E extends string> {
	private readonly eventListeners = new Map<E, ((arg: any) => void)[]>();

	public addEventListener(event: E, handler: (arg: any) => void): this {
		if (this.eventListeners.has(event))
			this.eventListeners.get(event)!.push(handler);
		else
			this.eventListeners.set(event, [ handler ]);
		return this;
	}

	public on(event: E, handler: (arg: any) => void): this {
		return this.addEventListener(event, handler);
	}

	/** @ignore */
	public emit(event: E, payload: any): this {
		if (this.eventListeners.has(event))
			this.eventListeners.get(event)!.forEach(fn => fn(payload));
		return this;
	}
}

class Child {
	constructor(
		/** The child process `pid`. */
		public readonly pid: number
	) {}

	/**
	 * Wrties `data` to the `stdin`.
	 *
	 * @param data The message to write, etiher a string or a byte array.
	 * @example
	 * ```typescript
	 * const command = new Command('node');
	 * const child = await command.spawn();
	 * await child.write('message');
	 * await child.write([ 0, 1, 2, 3, 4, 5 ]);
	 * ```
	 * @returns A Promise indicating the success or failure of the operation.
	 * An error can occur if the child process has exited.
	 */
	public async write(data: string | Uint8Array): Promise<void> {
		return await invokeMillenniumCommand<void>({
			__millenniumModule: 'Shell',
			message: {
				cmd: 'stdinWrite',
				pid: this.pid,
				buffer: typeof data === 'string' ? data : Array.from(data)
			}
		});
	}

	/**
	 * Kills the child process.
	 *
	 * @returns A Promise indicating the success or failure of the operation.
	 */
	public async kill(): Promise<void> {
		return await invokeMillenniumCommand<void>({
			__millenniumModule: 'Shell',
			message: {
				cmd: 'killChild',
				pid: this.pid
			}
		});
	}
}

/**
 * The entry point for spawning child processes. It emits `close` and `error` events.
 *
 * @example
 * ```typescript
 *
 * ```
 */
export class Command extends EventEmitter<'close' | 'error'> {
	private readonly args: string[];
	protected options: InternalSpawnOptions;
	public readonly stdout = new EventEmitter<'data'>();
	public readonly stderr = new EventEmitter<'data'>();

	/**
	 * Creates a new `Command` instance.
	 *
	 * @param program The program name to execute. It must be configured in `.millenniumrc > millennium > allowlist > shell > scope`.
	 */
	public constructor(private readonly program: string, args: string | string[] = [], options: SpawnOptions = {}) {
		super();
		this.args = typeof args === 'string' ? [ args ] : args;
		this.options = options;
	}

	/**
	 * Creates a `Command` to execute the given sidecar program.
	 *
	 * @example
	 * ```typescript
	 * const command = Command.sidecar('my-sidecar');
	 * const output = await command.execute();
	 * ```
	 *
	 * @param program The sidecar program name to execute. It must be configured in `.millenniumrc > millennium > allowlist > shell > scope`.
	 */
	public static sidecar(program: string, args: string | string[] = [], options?: SpawnOptions): Command {
		const instance = new Command(program, args, options);
		instance.options.sidecar = true;
		return instance;
	}

	/**
	 * Executes the command as a child process, returning a handle to it.
	 *
	 * @return A Promise resolving to the child process handle.
	 */
	public async spawn(): Promise<Child> {
		const eventHandler = ({ event, payload }: CommandEvent) => {
			switch (event) {
				case 'Error':
					this.emit('error', payload); break;
				case 'Terminated':
					this.emit('close', payload); break;
				case 'Stdout':
					this.stdout.emit('data', payload); break;
				case 'Stderr':
					this.stderr.emit('data', payload); break;
			}
		};
		const pid = await execute(eventHandler, this.program, this.args, this.options);
		return new Child(pid);
	}

	/**
	 * Executes the command as a child process, waiting for it to exit and collecting all of its output.
	 *
	 * @example
	 * ```typescript
	 * const output = await new Command('echo', 'message').execute();
	 * assert(output.code === 0);
	 * assert(output.signal === null);
	 * assert(output.stdout.trim() === 'message');
	 * assert(output.stderr === '');
	 * ```
	 */
	public execute(): Promise<ChildProcess> {
		return new Promise((resolve, reject) => {
			this.on('error', reject);

			const stdout: string[] = [];
			const stderr: string[] = [];
			this.stdout.on('data', (line: string) => stdout.push(line));
			this.stderr.on('data', (line: string) => stderr.push(line));

			this.on('close', (payload: TerminatedPayload) => {
				resolve({
					code: payload.code,
					signal: payload.signal,
					stdout: stdout.join('\n'),
					stderr: stderr.join('\n')
				});
			});
			this.spawn().catch(reject);
		});
	}
}

interface Event<T, V> {
	event: T;
	payload: V;
}

interface TerminatedPayload {
	/** Exit code of the process. `null` if the process was terminated by a signal on Unix. */
	code: number | null;
	/** If the process was terminated by a signal, this represents that signal. */
	signal: number | null;
}

type CommandEvent =
	| Event<'Stdout', string>
	| Event<'Stderr', string>
	| Event<'Terminated', TerminatedPayload>
	| Event<'Error', string>;

/**
 * Opens a path or URL with the system's default app for the file type, or the one specified with `openWith`.
 *
 * @example
 * ```typescript
 * // opens the given URL in the default browser:
 * await open('https://pyke.io/');
 * // opens the given URL in Firefox:
 * await open('https://pyke.io/', 'firefox');
 * // opens a file using the default program:
 * await open('/path/to/file.txt');
 * ```
 *
 * @param path The path or URL to open. This value is matched against the string regex defined in
 * `.millenniumrc > millennium > allowlist > shell > open`, which defaults to `^https?://`.
 * @param openWith The app to open the file or URL with. Defaults to the system default application for the specified path type.
 */
export async function open(
	path: string,
	openWith?:
		| 'firefox'
		| 'google chrome'
		| 'chromium'
		| 'safari'
		| 'open'
		| 'start'
		| 'xdg-open'
		| 'gio'
		| 'gnome-open'
		| 'kde-open'
		| 'wslview'
): Promise<void> {
	return await invokeMillenniumCommand({
		__millenniumModule: 'Shell',
		message: {
			cmd: 'open',
			path,
			with: openWith
		}
	});
}

export async function exec(
	program: string,
	args: string | string[] = [],
	options: SpawnOptions = {}
): Promise<ChildProcess> {
	return await new Command(program, args, options).execute();
}

export async function execSidecar(
	program: string,
	args: string | string[] = [],
	options: SpawnOptions = {}
): Promise<ChildProcess> {
	return await Command.sidecar(program, args, options).execute();
}

export async function spawn(
	program: string,
	args: string | string[] = [],
	options: SpawnOptions = {}
): Promise<Child> {
	return await new Command(program, args, options).spawn();
}

export async function spawnSidecar(
	program: string,
	args: string | string[] = [],
	options: SpawnOptions = {}
): Promise<Child> {
	return await Command.sidecar(program, args, options).spawn();
}

export async function showItemInFolder(path: string): Promise<void> {
	if (isWindows())
		// TODO: this only opens via explorer.exe, not [Files](https://files.community) if the user has it configured.
		// AFAIK Files doesn't even currently support showing a file in a folder, but when that comes,
		// we should really be using win32 `SHOpenFolderAndSelectItems` or
		// `ShellExecute(NULL, L"open", dir, NULL, NULL, SW_SHOW)`.
		await new Command('explorer', [ '/select,', path ]).execute();
	else if (isLinux())
		// TODO: this just opens the first implementation of org.freedesktop.FileManager1
		// most of the time this is the default file manager, but only *most* of the time.
		await new Command('dbus-send', [
			'--session',
			'--print-reply',
			'--dest=org.freedesktop.FileManager1',
			'--type=method_call',
			'/org/freedesktop/FileManager1',
			'org.freedesktop.FileManager1.ShowItems',
			`array:string:"file://${path}"`,
			'string:""'
		]).execute();
	else if (isMacOS())
		// thankfully, macOS's method is very simple :)
		await new Command('open', [ '-R', path ]).execute();
	else
		throw new Error('Unsupported platform');
}
