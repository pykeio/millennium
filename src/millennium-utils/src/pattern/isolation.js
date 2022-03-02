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
 * IMPORTANT: See ipc.js for the main frame implementation.
 * main frame -> isolation frame = isolation payload
 * isolation frame -> main frame = isolation message
 */

;(async function () {
	/**
	 * Sends a message to the isolation frame.
	 * @param {any} message
	 */
	function sendMessage(message) {
		window.parent.postMessage(message, '*');
	}

	const aesGcmKeyRaw = new Uint8Array(__TEMPLATE_runtime_aes_gcm_key__);

	const aesGcmKey = await window.crypto.subtle.importKey('raw', aesGcmKeyRaw, 'AES-GCM', true, [ 'encrypt' ]);

	/**
	 * @param {object} data
	 * @return {Promise<{nonce: number[], payload: number[]}>}
	 */
	async function encrypt(data) {
		const algorithm = Object.create(null);
		algorithm.name = 'AES-GCM';
		algorithm.iv = window.crypto.getRandomValues(new Uint8Array(12));

		const encoder = new TextEncoder();
		const payloadRaw = encoder.encode(JSON.stringify(data));

		return window.crypto.subtle
			.encrypt(algorithm, aesGcmKey, payloadRaw)
			.then(payload => {
				const result = Object.create(null);
				result.nonce = Array.from(new Uint8Array(algorithm.iv));
				result.payload = Array.from(new Uint8Array(payload));
				return result;
			});
	}

	/**
	 * Detect if a message event is a valid isolation payload.
	 *
	 * @param {MessageEvent<object>} event - a message event that is expected to be an isolation payload
	 * @return boolean
	 */
	function isIsolationPayload(event) {
		return typeof event.data === 'object' && 'callback' in event.data && 'error' in event.data;
	}

	/**
	 * Handle incoming payload events.
	 *
	 * @param {MessageEvent<any>} event
	 */
	async function payloadHandler(event) {
		if (!isIsolationPayload(event))
			return;

		let { data } = event;
		if (typeof window.__MILLENNIUM_ISOLATION_HOOK__ === 'function')
			// await even if it's not async so that we can support async hooks
			data = await window.__MILLENNIUM_ISOLATION_HOOK__(data);

		const encrypted = await encrypt(data);
		sendMessage(encrypted);
	}

	window.addEventListener('message', payloadHandler, false);

	/** @type {number} - How many milliseconds to wait between ready checks */
	const readyIntervalMs = 50

	/**
	 * Wait until this Isolation context is ready to receive messages, and let the main frame know.
	 */
	function waitUntilReady() {
		// consider either a function or an explicitly set null value as the ready signal
		if (typeof window.__MILLENNIUM_ISOLATION_HOOK__ === 'function' || window.__MILLENNIUM_ISOLATION_HOOK__ === null)
			sendMessage('__MILLENNIUM_ISOLATION_READY__');
		else
			setTimeout(waitUntilReady, readyIntervalMs);
	}

	setTimeout(waitUntilReady, readyIntervalMs);
})();
