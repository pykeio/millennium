/*
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

#include <Carbon/Carbon.h>

#include "carbon_hotkey_binding.h"

HotkeyCallback saved_callback = NULL;
void *saved_closure = NULL;

int hotkey_handler(EventHandlerCallRef next_handler, EventRef event, void *user_data) {
	(void)(next_handler);
	(void)(user_data);

	EventHotKeyID event_hotkey;

	int result = GetEventParameter(event, kEventParamDirectObject, typeEventHotKeyID, NULL, sizeof(event_hotkey), NULL, &event_hotkey);
	if (result == noErr && saved_callback && saved_closure)
		saved_callback(event_hotkey.id, saved_closure);
	return noErr;
}

void *install_event_handler(HotkeyCallback callback, void *data) {
	if (!callback || !data)
		return NULL;

	saved_callback = callback;
	saved_closure = data;
	
	EventTypeSpec event_type;
	event_type.eventClass = kEventClassKeyboard;
	event_type.eventKind = kEventHotKeyPressed;
	
	EventHandlerRef handler_ref;
	int result = InstallEventHandler(GetApplicationEventTarget(), &hotkey_handler, 1, &event_type, data, &handler_ref);
	if (result == noErr)
		return handler_ref;

	return NULL;
}

int uninstall_event_handler(void *handler_ref) {
	return RemoveEventHandler(handler_ref);
}

void *register_hotkey(int id, int modifier, int key) {
	EventHotKeyRef hotkey_ref;
	EventHotKeyID hotkey_id;
	hotkey_id.signature = 'htrs';
	hotkey_id.id = id;
	int result = RegisterEventHotKey(key, modifier, hotkey_id, GetApplicationEventTarget(), 0, &hotkey_ref);
	if (result == noErr)
		return hotkey_ref;

	return NULL;
}

int unregister_hotkey(void *hotkey_ref) {
	return UnregisterEventHotKey(hotkey_ref);
}
