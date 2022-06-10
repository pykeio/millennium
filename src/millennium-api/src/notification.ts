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
 * Send toast notifications (brief auto-expiring OS window element) to your user.
 * Can also be used via the web Notifications API.
 *
 * The APIs must be allowlisted in `.millenniumrc`:
 * ```json
 * {
 * 	"millennium": {
 * 		"allowlist": {
 * 			"notification": {
 * 				"all": true
 * 			}
 * 		}
 * 	}
 * }
 * ```
 *
 * @module
 */

import { invokeMillenniumCommand } from './helpers/millennium';

export interface NotificationOptions {
	title: string;
	body?: string;
	icon?: string;
}

export type NotificationPermissionStatus = 'granted' | 'denied' | 'default';

/**
 * Checks if the permission to send notifications is granted.
 */
export async function isPermissionGranted(): Promise<boolean> {
	if (window.Notification.permission !== 'default')
		return Promise.resolve(window.Notification.permission === 'granted');

	return invokeMillenniumCommand({
		__millenniumModule: 'Notification',
		message: {
			cmd: 'isNotificationPermissionGranted'
		}
	});
}

/**
 * Requests the permission to send notifications.
 */
export async function requestPermission(): Promise<NotificationPermissionStatus> {
	return window.Notification.requestPermission();
}

/**
 * Sends a notification to the user.
 *
 * @param options Either the notification title as a string or a `NotificationOptions` object.
 */
export function sendNotification(options: NotificationOptions | string): Notification {
	if (typeof options === 'string')
		return new window.Notification(options);
	else
		return new window.Notification(options.title, options);
}
