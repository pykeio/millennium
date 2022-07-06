// Copyright 2022 pyke.io
//           2019-2021 Tauri Programme within The Commons Conservancy
//                     [https://tauri.studio/]
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused_imports)]

use millennium_macros::{command_enum, module_command_handler, CommandModule};
use serde::Deserialize;

use super::InvokeContext;
use crate::Runtime;
#[cfg(notification_all)]
use crate::{api::notification::Notification, Env, Manager};

// `Granted` response from `request_permission`. Matches the Web API return
// value.
const PERMISSION_GRANTED: &str = "granted";
// `Denied` response from `request_permission`. Matches the Web API return
// value.
const PERMISSION_DENIED: &str = "denied";

/// The options for the notification API.
#[derive(Debug, Clone, Deserialize)]
pub struct NotificationOptions {
	/// The notification title.
	pub title: String,
	/// The notification body.
	pub body: Option<String>,
	/// The notification icon.
	pub icon: Option<String>
}

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
	/// The show notification API.
	#[cmd(notification_all, "notification > all")]
	Notification { options: NotificationOptions },
	/// The request notification permission API.
	RequestNotificationPermission,
	/// The notification permission check API.
	IsNotificationPermissionGranted
}

impl Cmd {
	#[module_command_handler(notification_all)]
	fn notification<R: Runtime>(context: InvokeContext<R>, options: NotificationOptions) -> super::Result<()> {
		let mut notification = Notification::new(context.config.millennium.bundle.identifier.clone()).title(options.title);
		if let Some(body) = options.body {
			notification = notification.body(body);
		}
		if let Some(icon) = options.icon {
			notification = notification.icon(icon);
		}
		#[cfg(feature = "windows7-compat")]
		notification.notify(&context.window.app_handle)?;
		#[cfg(not(feature = "windows7-compat"))]
		notification.show()?;
		Ok(())
	}

	fn request_notification_permission<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
		Ok(if cfg!(notification_all) { PERMISSION_GRANTED } else { PERMISSION_DENIED })
	}

	fn is_notification_permission_granted<R: Runtime>(_context: InvokeContext<R>) -> super::Result<bool> {
		Ok(cfg!(notification_all))
	}
}

#[cfg(test)]
mod tests {
	use quickcheck::{Arbitrary, Gen};

	use super::NotificationOptions;

	impl Arbitrary for NotificationOptions {
		fn arbitrary(g: &mut Gen) -> Self {
			Self {
				title: String::arbitrary(g),
				body: Option::arbitrary(g),
				icon: Option::arbitrary(g)
			}
		}
	}

	#[test]
	fn request_notification_permission() {
		assert_eq!(
			super::Cmd::request_notification_permission(crate::test::mock_invoke_context()).unwrap(),
			if cfg!(notification_all) { super::PERMISSION_GRANTED } else { super::PERMISSION_DENIED }
		)
	}

	#[test]
	fn is_notification_permission_granted() {
		assert_eq!(super::Cmd::is_notification_permission_granted(crate::test::mock_invoke_context()).unwrap(), cfg!(notification_all));
	}

	#[millennium_macros::module_command_test(notification_all, "notification > all")]
	#[quickcheck_macros::quickcheck]
	fn notification(_options: NotificationOptions) {}
}
