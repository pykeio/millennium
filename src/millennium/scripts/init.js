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
// @ts-nocheck

;(function () {
	if (window.location.origin.startsWith(__TEMPLATE_origin__)) {
		__RAW_freeze_prototype__

		;(function() {
			__RAW_hotkeys__
		})();

		__RAW_pattern_script__

		__RAW_ipc_script__

		;(function () {
			__RAW_bundle_script__
		})();

		__RAW_listen_function__

		__RAW_core_script__

		__RAW_event_initialization_script__

		if (window.ipc)
			window.__MILLENNIUM_INVOKE__('__initialized', { url: window.location.href });
		else
			window.addEventListener('DOMContentLoaded', function () {
				window.__MILLENNIUM_INVOKE__('__initialized', { url: window.location.href });
			});

		__RAW_plugin_initialization_script__
	}
})();
