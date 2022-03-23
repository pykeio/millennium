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
 * Provides APIs to create windows, communicate with other windows and manipulate the current window.
 *
 * This package is also accessible with `window.Millennium.window` when `.millenniumrc > build > withGlobalMillennium` is set to true.
 *
 * The APIs must be allowlisted on `.millenniumrc`:
 * ```json
 * {
 * 	"millennium": {
 * 		"allowlist": {
 * 			"window": {
 * 				"all": true, // enable all window APIs
 * 				"create": true, // enable window creation
 * 				"center": true,
 * 				"requestUserAttention": true,
 * 				"setResizable": true,
 * 				"setTitle": true,
 * 				"maximize": true,
 * 				"unmaximize": true,
 * 				"minimize": true,
 * 				"unminimize": true,
 * 				"show": true,
 * 				"hide": true,
 * 				"close": true,
 * 				"setDecorations": true,
 * 				"setAlwaysOnTop": true,
 * 				"setSize": true,
 * 				"setMinSize": true,
 * 				"setMaxSize": true,
 * 				"setPosition": true,
 * 				"setFullscreen": true,
 * 				"setFocus": true,
 * 				"setIcon": true,
 * 				"setSkipTaskbar": true,
 * 				"startDragging": true,
 * 				"print": true
 * 			}
 * 		}
 * 	}
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 *
 * # Window events
 *
 * Events can be listened using `appWindow.listen`:
 * ```typescript
 * import { appWindow } from '@pyke/millennium-api/window';
 * appWindow.listen('millennium://move', ({ event, payload }) => {
 * 	const { x, y } = payload; // payload here is a `PhysicalPosition`
 * });
 * ```
 *
 * Window-specific events emitted by the backend:
 *
 * #### 'millennium://resize'
 * Emitted when the size of the window has changed.
 * *EventPayload*:
 * ```typescript
 * type ResizePayload = PhysicalSize;
 * ```
 *
 * #### 'millennium://move'
 * Emitted when the position of the window has changed.
 * *EventPayload*:
 * ```typescript
 * type MovePayload = PhysicalPosition;
 * ```
 *
 * #### 'millennium://close-requested'
 * Emitted when the user requests the window to be closed.
 * If a listener is registered for this event, Millennium won't close the window so you must call `appWindow.close()` manually.
 *
 * #### 'millennium://focus'
 * Emitted when the window gains focus.
 *
 * #### 'millennium://blur'
 * Emitted when the window loses focus.
 *
 * #### 'millennium://scale-change'
 * Emitted when the window's scale factor has changed.
 * The following user actions can cause DPI changes:
 * - Changing the display's resolution.
 * - Changing the display's scale factor (e.g. in Control Panel on Windows).
 * - Moving the window to a display with a different scale factor.
 * *Event payload*:
 * ```typescript
 * interface ScaleFactorChanged {
 * 	scaleFactor: number;
 * 	size: PhysicalSize;
 * }
 * ```
 *
 * #### 'millennium://menu'
 * Emitted when a menu item is clicked.
 * *EventPayload*:
 * ```typescript
 * type MenuClicked = string;
 * ```
 *
 * @module
 */

import { invokeMillenniumCommand } from './helpers/millennium';
import { emit, listen, once, EventName, EventCallback, Unlistener } from './event';
import { isBrowser } from './platform';

export interface Monitor {
	name: string | null;
	/** The monitor's resolution. */
	size: PhysicalSize;
	/** The top left corner position of the monitor relative to the larger full screen area. */
	position: PhysicalPosition;
	/** The scale factor used to map physical pixels to logical pixels. Commonly also called device pixel ratio. */
	scaleFactor: number;
}

export class LogicalSize {
	public readonly TYPE = 'Logical';

	constructor(public width: number, public height: number) {}
}

export class PhysicalSize {
	public readonly TYPE = 'Physical';

	constructor(public width: number, public height: number) {}

	public toLogical(scaleFactor: number): LogicalSize {
		return new LogicalSize(this.width / scaleFactor, this.height / scaleFactor);
	}
}

export class LogicalPosition {
	public readonly TYPE = 'Logical';

	constructor(public x: number, public y: number) {}
}

export class PhysicalPosition {
	public readonly TYPE = 'Physical';

	constructor(public x: number, public y: number) {}

	public toLogical(scaleFactor: number): LogicalPosition {
		return new LogicalPosition(this.x / scaleFactor, this.y / scaleFactor);
	}
}

function validatePosition(position: LogicalPosition | PhysicalPosition) {
	if (!position || (position.TYPE !== 'Logical' && position.TYPE !== 'Physical'))
		throw new Error('Invalid position! Must be an instance of `LogicalPosition` or `PhysicalPosition`.');
}

function validateSize(size: LogicalSize | PhysicalSize) {
	if (!size || (size.TYPE !== 'Logical' && size.TYPE !== 'Physical'))
		throw new Error('Invalid size! Must be an instance of `LogicalSize` or `PhysicalSize`.');
}

interface WindowDef {
	label: string;
}

declare global {
	interface Window {
		__MILLENNIUM_METADATA__: {
			__windows: WindowDef[];
			__currentWindow: WindowDef;
		}
	}
}

export enum UserAttentionType {
	/**
	 * ## Platform-specific
	 * - **macOS**: Bounces the dock icon until the application is in focus.
	 * - **Windows**: Flashes both the window and the taskbar button until the application is in focus.
	 */
	CRITICAL = 1,
	/**
	 * ## Platform-specific
	 * - **macOS**: Bounces the dock icon once.
	 * - **Windows**: Flashes the taskbar button until the application is in focus.
	 */
	INFORMATIONAL = 2
}

/**
 * Get an instance of `WebviewWindow` for the current webview window.
 */
export function getCurrentWindow(): WebviewWindow {
	return new WebviewWindow(window.__MILLENNIUM_METADATA__.__currentWindow.label, { skip: true });
}

/**
 * Gets an instance of `WebviewWindow` for all available webview windows.
 */
export function getAllWindows(): WebviewWindow[] {
	return window.__MILLENNIUM_METADATA__.__windows.map(({ label }) => new WebviewWindow(label, { skip: true }));
}

const localMillenniumEvents = [ 'millennium://created', 'millennium://error' ];
export type WindowLabel = string;

/**
 * A webview window handle allows emitting and listening to events from the backend that are tied to the window.
 */
class WebviewWindowHandle {
	private readonly listeners = new Map<string, Array<EventCallback<any>>>();

	protected constructor(public readonly label: WindowLabel) {}

	/**
	 * Listen to an event emitted by the backend that is tied to the webview window.
	 */
	public async listen<T>(event: EventName, handler: EventCallback<T>): Promise<Unlistener> {
		if (this.handleMillenniumEvent(event, handler)) {
			const listeners = this.listeners.get(event);
			listeners?.splice(listeners.indexOf(handler), 1);
			return () => {};
		}

		return await listen(event, this.label, handler);
	}

	/**
	 * Listen to a one-off event emitted by the backend that is tied to the webview window.
	 */
	public async once<T>(event: EventName, handler: EventCallback<T>): Promise<Unlistener> {
		if (this.handleMillenniumEvent(event, handler)) {
			const listeners = this.listeners.get(event);
			listeners!.splice(listeners!.indexOf(handler), 1);
			return () => {};
		}

		return await once(event, this.label, handler);
	}

	/**
	 * Emits an event to the backend, tied to the webview window.
	 */
	public async emit(event: string, payload?: unknown): Promise<void> {
		if (localMillenniumEvents.includes(event)) {
			for (const handler of this.listeners.get(event) ?? [])
				handler({ event, id: -1, windowLabel: this.label, payload });

			return;
		}

		return await emit(event, this.label, payload);
	}

	private handleMillenniumEvent<T>(event: string, handler: EventCallback<T>): boolean {
		if (localMillenniumEvents.includes(event)) {
			if (!this.listeners.has(event))
				this.listeners.set(event, [ handler ]);
			else
				this.listeners.get(event)!.push(handler);

			return true;
		}

		return false;
	}
}

class WindowManager extends WebviewWindowHandle {
	private async _manage<T>(type: string, payload?: unknown): Promise<T> {
		return await invokeMillenniumCommand<T>({
			__millenniumModule: 'Window',
			message: {
				cmd: 'manage',
				data: {
					label: this.label,
					cmd: {
						type,
						...(payload !== undefined ? { payload } : {})
					}
				}
			}
		});
	}

	/** The scale factor that can be used to map physical pixels to logical pixels. */
	public async scaleFactor(): Promise<number> {
		return await this._manage('scaleFactor');
	}

	/** The position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop. */
	public async innerPosition(): Promise<PhysicalPosition> {
		return await this._manage<{ x: number, y: number }>('position').then(({ x, y }) => new PhysicalPosition(x, y));
	}

	/** The position of the top-left hand corner of the window relative to the top-left hand corner of the desktop. */
	public async outerPosition(): Promise<PhysicalPosition> {
		return await this._manage<{ x: number, y: number }>('outerPosition').then(({ x, y }) => new PhysicalPosition(x, y));
	}

	/** The physical size of the window's client area. The client area is the content of the window, excluding the title bar and borders. */
	public async innerSize(): Promise<PhysicalSize> {
		return await this._manage<{ width: number, height: number }>('innerSize').then(({ width, height }) => new PhysicalSize(width, height));
	}

	/** The physical size of the entire window, including the title bar and borders. **You probably want `innerSize()`**. */
	public async outerSize(): Promise<PhysicalSize> {
		return await this._manage<{ width: number, height: number }>('outerSize').then(({ width, height }) => new PhysicalSize(width, height));
	}

	/** Get's the window's current fullscreen state. */
	public async isFullscreen(): Promise<boolean> {
		return await this._manage('isFullscreen');
	}

	/** Gets the window's current maximized state. */
	public async isMaximized(): Promise<boolean> {
		return await this._manage('isMaximized');
	}

	/** Gets the window's current decorated state (has titlebar, borders, shadow). */
	public async isDecorated(): Promise<boolean> {
		return await this._manage('isDecorated');
	}

	/** Gets the window's current resizable state. */
	public async isResizable(): Promise<boolean> {
		return await this._manage('isResizable');
	}

	/** Gets the window's current visibility state. */
	public async isVisible(): Promise<boolean> {
		return await this._manage('isVisible');
	}

	/** Centers the window on the display the window is currently on. */
	public async center(): Promise<void> {
		return await this._manage('center');
	}

	/**
	 * Requests user attention to the window. This has no effect if the application is already focused.
	 * How requesting for user attention manifests is platform-dependent, see `UserAttentionType` for platform specific
	 * details.
	 *
	 * Providing `null` will unset the request for user attention. Unsetting the request for user attention might not
	 * always be done automatically by the WM when the window receives input (Windows is particularly bad at this)
	 *
	 * ## Platform-specific
	 * - **macOS**: `null` has no effect, but is more reliable turning off attention when focus is gained.
	 * - **Linux**: Urgency levels have the same effect.
	 */
	async requestUserAttention(requestType: UserAttentionType | null = null): Promise<void> {
		return await this._manage('requestUserAttention', (
			requestType === UserAttentionType.CRITICAL
				? { type: 'Critical' }
				: requestType === UserAttentionType.INFORMATIONAL
					? { type: 'Informational' }
					: null
		));
	}

	/** Sets the window's resizable flag. */
	public async setResizable(resizable: boolean): Promise<void> {
		return await this._manage('setResizable', resizable);
	}

	/** Sets the window title. */
	public async setTitle(title: string): Promise<void> {
		return await this._manage('setTitle', title);
	}

	/** Maximizes the window. */
	public async maximize(): Promise<void> {
		return await this._manage('maximize');
	}

	/** Unmaximizes/restores the window. */
	public async unmaximize(): Promise<void> {
		return await this._manage('unmaximize');
	}

	/** Toggles the window's maximized state. */
	public async toggleMaximized(): Promise<void> {
		return await this._manage('toggleMaximized');
	}

	/** Minimizes the window. */
	public async minimize(): Promise<void> {
		return await this._manage('minimize');
	}

	/** Unminimizes/restores the window. */
	public async unminimize(): Promise<void> {
		return await this._manage('unminimize');
	}

	/** Sets the window to be visible. */
	public async show(): Promise<void> {
		return await this._manage('show');
	}

	/** Hides the window. */
	public async hide(): Promise<void> {
		return await this._manage('hide');
	}

	/** Closes the window. */
	public async close(): Promise<void> {
		return await this._manage('close');
	}

	/** Sets whether or not the window should have borders and bars. */
	async setDecorations(decorations: boolean): Promise<void> {
		return await this._manage('setDecorations', decorations);
	}

	/** Sets whether or not the window should be always on top of other windows. */
	async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
		return await this._manage('setAlwaysOnTop', alwaysOnTop);
	}

	/** Sets the (inner!) size of the window. */
	async setSize(size: LogicalSize | PhysicalSize): Promise<void> {
		validateSize(size);
		return await this._manage('setSize', {
			type: size.TYPE,
			data: {
				width: size.width,
				height: size.height
			}
		});
	}

	/** Sets the minimum inner size. If the `size` argument is not provided, the constraint is unset. */
	async setMinimumSize(size: LogicalSize | PhysicalSize | null = null): Promise<void> {
		if (size)
			validateSize(size);
		return await this._manage('setMinSize', size === null
			? null
			: {
				type: size.TYPE,
				data: {
					width: size.width,
					height: size.height
				}
			}
		);
	}

	/** Sets the maximum inner size. If the `size` argument is not provided, the constraint is unset. */
	async setMaximumSize(size: LogicalSize | PhysicalSize | null = null): Promise<void> {
		if (size)
			validateSize(size);
		return await this._manage('setMaxSize', size === null
			? null
			: {
				type: size.TYPE,
				data: {
					width: size.width,
					height: size.height
				}
			}
		);
	}

	/** Sets the window outer position. */
	async setPosition(position: LogicalPosition | PhysicalPosition): Promise<void> {
		validatePosition(position);
		return await this._manage('setPosition', {
			type: position.TYPE,
			data: {
				x: position.x,
				y: position.y
			}
		});
	}

	/** Sets the window's fullscreen state. */
	async setFullscreen(fullscreen: boolean): Promise<void> {
		return await this._manage('setFullscreen', fullscreen);
	}

	/** Bring the window to the front and focus. */
	async focus(): Promise<void> {
		return await this._manage('setFocus');
	}

	/**
	 * Sets the window icon.
	 *
	 * Note that you need the `icon-ico` or `icon-png` Cargo features to use this API.
	 * To enable it, change your Cargo.toml file:
	 * ```toml
	 * [dependencies]
	 * millennium = { version = "...", features = [ "...", "icon-png" ] }
	 * ```
	 *
	 * @param icon Raw icon RGBA bytes or path to an icon file.
	 */
	async setIcon(icon: string | Uint8Array): Promise<void> {
		return await this._manage('setIcon', {
			icon: typeof icon === 'string' ? icon : Array.from(icon)
		});
	}

	/** Whether to show the window icon in the taskbar or not. */
	async setShowInTaskbar(show: boolean = true): Promise<void> {
		return await this._manage('setSkipTaskbar', show);
	}

	/** Starts dragging the window. */
	async startDragging(): Promise<void> {
		return await this._manage('startDragging');
	}
}

/**
 * Create new webview windows and get handles to existing ones.
 *
 * Windows are identified by a label, a unique identifier that can be used to reference it later.
 * It may only contain alphanumeric characters (`a-zA-Z0-9`) and `-`, `/`, `:`, or `_`.
 *
 * @example
 * ```typescript
 * // loading embedded assets:
 * const webview = new WebviewWindow('super-unique-label', {
 * 	url: 'path/to/page.html'
 * });
 *
 * // alternatively, load a remote URL:
 * const webview = new WebviewWindow('super-unique-label', {
 * 	url: 'https://example.com/'
 * });
 *
 * // emit an event to the backend
 * await webview.emit('some-event', { ... });
 * // listen to an event from the backend
 * const unlisten = await webview.listen('some-other-event', e => { ... });
 * unlisten();
 * ```
 */
export class WebviewWindow extends WindowManager {
	/**
	 * Creates a new webview window.
	 *
	 * @param label The unique window label. Must be composed of only `a-zA-Z0-9-/:_`.
	 * @param options The window options.
	 */
	public constructor(label: WindowLabel, options: (WindowOptions & { /** @internal */ skip?: true }) = {}) {
		super(label);

		if (!options.skip) {
			invokeMillenniumCommand({
				__millenniumModule: 'Window',
				message: {
					cmd: 'createWebview',
					data: {
						options: {
							label,
							...options
						}
					}
				}
			})
				.then(async () => this.emit('millennium://created'))
				.catch(async (e: string) => this.emit('millennium://error', e));
		}
	}

	/**
	 * Gets the webview window associated with the given label.
	 *
	 * @param label The window label.
	 * @returns The webview window, or `null` if no window with the given label exists.
	 */
	public static getByLabel(label: string): WebviewWindow | null {
		if (getAllWindows().some(w => w.label === label))
			return new WebviewWindow(label, { skip: true });

		return null;
	}
}

/** A reference to the current webview window. */
let appWindow: WebviewWindow;
if (isBrowser() && '__MILLENNIUM_METADATA__' in window)
	appWindow = new WebviewWindow(window.__MILLENNIUM_METADATA__.__currentWindow.label, { skip: true });
else {
	console.warn(`Could not find __MILLENNIUM_METADATA__. The "appWindow" value will reference the window with the "main" label.
This is not an issue if you are running this frontend in a browser instead of a Millennium window.`);
	appWindow = new WebviewWindow('main', { skip: true });
}

export { appWindow };

export interface WindowOptions {
	/**
	 * Remote URL or local file path to open.
	 *
	 * This can be a:
	 * - URL such as `https://pyke.io`, which is opened directly in the window.
	 * - `data:` URI such as `data:text/html,<html>...` which is only supported with the `window-data-url` Cargo feature for the `millennium` crate.
	 * - local file path or route such as `/home/user/file.html`, which is appended to the application URL (the devServer URL in development, or `millennium://` & `https://millennium.localhost/` in production).
	 */
	url?: string;
	/** Show the window in the center of the display. */
	center?: boolean;
	/** The initial x position, **not** relative to the current display (see `currentMonitor`, `primaryMonitor`, & `availableMonitors`). Only applies if `y` is also set. */
	x?: number;
	/** The initial y position, **not** relative to the current display (see `currentMonitor`, `primaryMonitor`, & `availableMonitors`). Only applies if `x` is also set. */
	y?: number;
	/** The initial width. */
	width?: number;
	/** The initial height. */
	height?: number;
	/** The minimum width. Only applies if `minHeight` is also set. */
	minWidth?: number;
	/** The minimum height. Only applies if `minWidth` is also set. */
	minHeight?: number;
	/** The maximum width. Only applies if `maxHeight` is also set. */
	maxWidth?: number;
	/** The maximum height. Only applies if `maxWidth` is also set. */
	maxHeight?: number;
	resizable?: boolean;
	title?: string;
	fullscreen?: boolean;
	/** Whether the window will be initially focused. */
	focus?: boolean;
	/**
	 * Whether the window is transparent or not.
	 * Note that on macOS, this requires the `macos-private-api` feature flag, enabled under `.millenniumrc > millennium > macosPrivateApi`.
	 * **Using private APIs on macOS will make your application ineligible for the App Store.**
	 */
	transparent?: boolean;
	titlebarHidden?: boolean;
	maximized?: boolean;
	visibile?: boolean;
	decorations?: boolean;
	alwaysOnTop?: boolean;
	skipTaskbar?: boolean;
	fileDropEnabled?: boolean;
}

/**
 * Returns the monitor on which the window currently resides, or `null` if the current monitor can't be detected.
 */
export async function currentMonitor(): Promise<Monitor | null> {
	return await invokeMillenniumCommand({
		__millenniumModule: 'Window',
		message: {
			cmd: 'manage',
			data: {
				cmd: {
					type: 'currentMonitor'
				}
			}
		}
	});
}

/**
 * Returns the primary monitor, or `null` if the primary monitor can't be detected.
 */
export async function primaryMonitor(): Promise<Monitor | null> {
	return invokeMillenniumCommand({
		__millenniumModule: 'Window',
		message: {
			cmd: 'manage',
			data: {
				cmd: {
					type: 'primaryMonitor'
				}
			}
		}
	});
}

/**
 * Returns an array of all available monitors.
 */
export async function availableMonitors(): Promise<Monitor[]> {
	return await invokeMillenniumCommand({
		__millenniumModule: 'Window',
		message: {
			cmd: 'manage',
			data: {
				cmd: {
					type: 'availableMonitors'
				}
			}
		}
	});
}
