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

/// <reference path="./types.d.ts" />

;(function () {
	function uid() {
		return window.crypto.getRandomValues(new Uint32Array(1))[0];
	}

	if (!window.Millennium)
		Object.defineProperty(window, 'Millennium', {
			value: {},
			writable: true,
			configurable: false,
			enumerable: true
		});

	window.Millennium.transformCallback = function transformCallback(
		callback,
		once
	) {
		const identifier = uid();
		const prop = `_${identifier}`;

		Object.defineProperty(window, prop, {
			value: result => {
				if (once)
					delete window[prop];

				return callback && callback(result);
			},
			writable: true,
			enumerable: false,
			configurable: true
		});

		return identifier;
	};

	const ipcQueue = [];
	let isWaitingForIpc = false;

	function waitForIpc() {
		if ('__MILLENNIUM_IPC__' in window)
			for (const action of ipcQueue)
				action();
		else
			setTimeout(waitForIpc, 50);
	}

	window.__MILLENNIUM_INVOKE__ = function invoke(cmd, args = {}) {
		return new Promise(function (resolve, reject) {
			const callback = window.Millennium.transformCallback(r => {
				resolve(r);
				delete window[`_${error}`];
			}, true);
			const error = window.Millennium.transformCallback(e => {
				reject(new Error(e));
				delete window[`_${callback}`];
			}, true);

			if (typeof cmd === 'string')
				args.cmd = cmd;
			else if (typeof cmd === 'object')
				args = cmd;
			else
				return reject(new Error('Invalid argument type'));

			const action = () =>
				window.__MILLENNIUM_IPC__({
					...args,
					callback,
					error: error
				});

			if (window.__MILLENNIUM_IPC__)
				action();
			else {
				ipcQueue.push(action);
				if (!isWaitingForIpc) {
					waitForIpc();
					isWaitingForIpc = true;
				}
			}
		});
	}

	// open <a href="..."> links with the Millennium API
	function __openLinks() {
		document.querySelector('body').addEventListener('click', e => {
			/** @type {Omit<MouseEvent, 'target'> & { target: HTMLAnchorElement }} */
			let { target } = /** @type {any} */(e);
			while (target != null) {
				if (target.matches('a')) {
					if (target.href && target.href.startsWith('http')) {
						window.__MILLENNIUM_INVOKE__('millennium', {
							__millenniumModule: 'Shell',
							message: {
								cmd: 'open',
								path: target.href
							}
						});
						e.preventDefault();
					}

					break;
				}
				target = /** @type {HTMLAnchorElement} */(target.parentElement);
			}
		});
	}

	if (document.readyState === 'complete' || document.readyState === 'interactive')
		__openLinks();
	else
		window.addEventListener('DOMContentLoaded', () => __openLinks(), true);

	// drag region
	document.addEventListener('mousedown', e => {
		if (/** @type {HTMLElement} */(e.target).hasAttribute('data-app-drag-region') && e.buttons === 1) {
			// Prevents a text cursor from appearing when dragging
			e.preventDefault();

			// Start dragging if the element has an `app-drag-region` data attribute and maximize on double-clicking it
			window.__MILLENNIUM_INVOKE__('millennium', {
				__millenniumModule: 'Window',
				message: {
					cmd: 'manage',
					data: {
						cmd: {
							type: e.detail === 2 ? '__toggleMaximize' : 'startDragging'
						}
					}
				}
			});
		}
	});

	listen('millennium://window-created', function (event) {
		if (event.payload) {
			var windowLabel = event.payload.label
			window.__MILLENNIUM_METADATA__.__windows.push({
				label: windowLabel
			});
		}
	});

	let permissionSettable = false;
	/** @type {NotificationPermission} */
	let permissionValue = 'default';

	function isPermissionGranted() {
		if (window.Notification.permission !== 'default')
			return Promise.resolve(window.Notification.permission === 'granted');

		return window.__MILLENNIUM_INVOKE__('millennium', {
			__millenniumModule: 'Notification',
			message: {
				cmd: 'isNotificationPermissionGranted'
			}
		});
	}

	/** @param {NotificationPermission} value */
	function setNotificationPermission(value) {
		permissionSettable = true;
		// @ts-ignore
		window.Notification.permission = value;
		permissionSettable = false;
	}

	function requestPermission() {
		return window.__MILLENNIUM_INVOKE__('millennium', {
			__millenniumModule: 'Notification',
			message: {
				cmd: 'requestNotificationPermission'
			}
		})
			.then(/** @param {NotificationPermission} permission */ permission => {
				setNotificationPermission(permission);
				return permission;
			});
	}

	/** @param {(NotificationOptions & { title: string }) | string} options */
	function sendNotification(options) {
		if (typeof options === 'object')
			Object.freeze(options);

		return window.__MILLENNIUM_INVOKE__('millennium', {
			__millenniumModule: 'Notification',
			message: {
				cmd: 'notification',
				options:
					typeof options === 'string'
						? { title: options }
						: options
			}
		});
	}

	// @ts-ignore
	window.Notification = function (title, options = {}) {
		sendNotification(Object.assign(options, { title }));
	};

	window.Notification.requestPermission = requestPermission;

	Object.defineProperty(window.Notification, 'permission', {
		enumerable: true,
		get: function() {
			return permissionValue;
		},
		set: function(v) {
			if (!permissionSettable)
				throw new TypeError('Attempted to assign to readonly property');

			permissionValue = v;
		}
	});

	isPermissionGranted().then(function (response) {
		if (response === null)
			setNotificationPermission('default');
		else
			setNotificationPermission(response ? 'granted' : 'denied');
	});

	window.alert = function (message) {
		window.__MILLENNIUM_INVOKE__('millennium', {
			__millenniumModule: 'Dialog',
			message: {
				cmd: 'messageDialog',
				message: message
			}
		});
	};

	// @ts-ignore
	window.confirm = function (message) {
		return window.__MILLENNIUM_INVOKE__('millennium', {
			__millenniumModule: 'Dialog',
			message: {
				cmd: 'confirmDialog',
				message: message
			}
		});
	};

	// window.print works on Linux/Windows; need to use the API on macOS
	if (/macintosh/i.test(navigator.userAgentData ? navigator.userAgentData.platform : navigator.userAgent))
		window.print = function() {
			return window.__MILLENNIUM_INVOKE__('millennium', {
				__millenniumModule: 'Window',
				message: {
					cmd: 'manage',
					data: {
						cmd: {
							type: 'print'
						}
					}
				}
			});
		};
})();
