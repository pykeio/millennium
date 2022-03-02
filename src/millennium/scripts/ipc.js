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
	/** @type {string} */
	const pattern = window.__MILLENNIUM_PATTERN__.pattern

	/** @type {string} */
	// @ts-ignore
	const isolationOrigin = __TEMPLATE_isolation_origin__

	/**
	 * @type {{ queue: object[], ready: boolean, frame: HTMLIFrameElement | null }}
	 */
	const isolation = Object.create(null);
	isolation.queue = [];
	isolation.ready = false;
	isolation.frame = null;

	/**
	 * Detects if a message event is a valid isolation message.
	 *
	 * @param {MessageEvent<object>} event - a message event that is expected to be an isolation message
	 * @return {boolean} - if the event was a valid isolation message
	 */
	function isIsolationMessage(event) {
		return typeof event.data === 'object' && 'nonce' in event.data && 'payload' in event.data;
	}

	/**
	 * Detects if data is able to transform into an isolation payload.
	 *
	 * @param {object} data - object that is expected to contain at least a callback and error identifier
	 * @return {boolean} - if the data is able to transform into an isolation payload
	 */
	function isIsolationPayload(data) {
		return typeof data === 'object' && 'callback' in data && 'error' in data;
	}

	/**
	 * Sends a properly formatted message to the isolation frame.
	 *
	 * @param {MillenniumIsolationPayload} data - data that has been validated to be an isolation payload
	 */
	function sendIsolationMessage(data) {
		// set the frame dom element if it's not been set before
		if (!isolation.frame) {
			/** @type {HTMLIFrameElement} */
			const frame = document.querySelector('iframe#__millennium_isolation__')
			if (frame.src.startsWith(isolationOrigin))
				isolation.frame = frame;
			else
				console.error('Millennium IPC found an isolation iframe, but it had the wrong origin');
		}

		// ensure we have the target to send the message to
		if (!isolation.frame || !isolation.frame.contentWindow) {
			console.error('Millennium isolation pattern could not find the isolation iframe window.');
			return;
		}

		isolation.frame.contentWindow.postMessage(data, '*' /* todo: set this to the secure origin */);
	}

	Object.defineProperty(window, '__MILLENNIUM_IPC__', {
		value: Object.freeze(message => {
			switch (pattern) {
				case 'brownfield':
					window.__MILLENNIUM_POST_MESSAGE__(message);
					break;
				case 'isolation':
					if (!isIsolationPayload(message)) {
						console.error('Millennium isolation pattern found an invalid isolation message payload', message);
						break;
					}

					if (isolation.ready)
						sendIsolationMessage(message)
					else
						isolation.queue.push(message);

					break;
				case 'error':
				default:
					console.error('Millennium IPC found a Millennium pattern, but it was an error type. Check for other log messages to find the cause.');
					break;
			}
		})
	});

	/**
	 * IMPORTANT: See isolation_secure.js for the isolation frame implementation.
	 * main frame -> isolation frame = isolation payload
	 * isolation frame -> main frame = isolation message
	 */
	if (pattern === 'isolation')
		window.addEventListener('message', event => {
			// watch for the isolation frame being ready and flush any queued messages
			if (event.data === '__MILLENNIUM_ISOLATION_READY__') {
				isolation.ready = true;

				for (const message of isolation.queue)
					sendIsolationMessage(message);

				isolation.queue = [];
				return;
			}

			if (isIsolationMessage(event))
				window.__MILLENNIUM_POST_MESSAGE__(event.data);
		}, false);
})();
