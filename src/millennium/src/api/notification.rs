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

#![allow(clippy::tabs_in_doc_comments)]

//! Types and functions related to desktop notifications.

#[cfg(windows)]
use std::path::MAIN_SEPARATOR;

/// The desktop notification definition.
///
/// Allows you to construct a Notification data and send it.
///
/// # Examples
/// ```rust,no_run
/// use millennium::api::notification::Notification;
/// // first we build the application to access the Millennium configuration
/// let app = millennium::Builder::default()
/// 	// on an actual app, remove the string argument
/// 	.build(millennium::generate_context!("test/fixture/.millenniumrc"))
/// 	.expect("error while building Millennium application");
///
/// // shows a notification with the given title and body
/// Notification::new(&app.config().millennium.bundle.identifier)
/// 	.title("New message")
/// 	.body("You've got a new message.")
/// 	.show();
///
/// // run the app
/// app.run(|_app_handle, _event| {});
/// ```
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Notification {
	/// The notification body.
	body: Option<String>,
	/// The notification title.
	title: Option<String>,
	/// The notification icon.
	icon: Option<String>,
	/// The notification identifier
	identifier: String
}

impl Notification {
	/// Initializes a instance of a Notification.
	pub fn new(identifier: impl Into<String>) -> Self {
		Self {
			identifier: identifier.into(),
			..Default::default()
		}
	}

	/// Sets the notification body.
	#[must_use]
	pub fn body(mut self, body: impl Into<String>) -> Self {
		self.body = Some(body.into());
		self
	}

	/// Sets the notification title.
	#[must_use]
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}

	/// Sets the notification icon.
	#[must_use]
	pub fn icon(mut self, icon: impl Into<String>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Shows the notification.
	///
	/// # Examples
	///
	/// ```no_run
	/// use millennium::api::notification::Notification;
	///
	/// // on an actual app, remove the string argument
	/// let context = millennium::generate_context!("test/fixture/.millenniumrc");
	/// Notification::new(&context.config().millennium.bundle.identifier)
	/// 	.title("Millennium App")
	/// 	.body("Test notification")
	/// 	.show()
	/// 	.unwrap();
	/// ```
	///
	/// ## Platform-specific
	///
	/// - **Windows**: Not supported on Windows 7. If your app targets it, enable the `windows7-compat` feature and use
	///   [`Self::notify`].
	#[cfg_attr(
		all(not(doc_cfg), feature = "windows7-compat"),
		deprecated = "This function does not work on Windows 7. Use `Self::notify` instead."
	)]
	pub fn show(self) -> crate::api::Result<()> {
		let mut notification = notify_rust::Notification::new();
		if let Some(body) = self.body {
			notification.body(&body);
		}
		if let Some(title) = self.title {
			notification.summary(&title);
		}
		if let Some(icon) = self.icon {
			notification.icon(&icon);
		} else {
			notification.auto_icon();
		}
		#[cfg(windows)]
		{
			let exe = millennium_utils::platform::current_exe()?;
			let exe_dir = exe.parent().expect("failed to get exe directory");
			let curr_dir = exe_dir.display().to_string();
			// set the notification's System.AppUserModel.ID only when running the installed
			// app
			if !(curr_dir.ends_with(format!("{S}target{S}debug", S = MAIN_SEPARATOR).as_str())
				|| curr_dir.ends_with(format!("{S}target{S}release", S = MAIN_SEPARATOR).as_str()))
			{
				notification.app_id(&self.identifier);
			}
		}
		#[cfg(target_os = "macos")]
		{
			let _ = notify_rust::set_application(if cfg!(feature = "custom-protocol") { &self.identifier } else { "com.apple.Terminal" });
		}

		crate::async_runtime::spawn(async move {
			let _ = notification.show();
		});

		Ok(())
	}

	/// Shows the notification. This API is similar to [`Self::show`], except it supports Windows 7.
	///
	/// # Examples
	///
	/// ```no_run
	/// use millennium::api::notification::Notification;
	///
	/// // on an actual app, remove the string argument
	/// let context = millennium::generate_context!("test/fixture/.millenniumrc");
	/// let identifier = context.config().millennium.bundle.identifier.clone();
	///
	/// millennium::Builder::default()
	/// 	.setup(move |app| {
	/// 		Notification::new(&identifier)
	/// 			.title("Millennium App")
	/// 			.body("Test notification")
	/// 			.notify(&app.handle())
	/// 			.unwrap();
	/// 		Ok(())
	/// 	})
	/// 	.run(context)
	/// 	.expect("error while running millennium application");
	/// ```
	#[cfg(feature = "windows7-compat")]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "windows7-compat")))]
	#[allow(unused_variables)]
	pub fn notify<R: crate::Runtime>(self, app: &crate::AppHandle<R>) -> crate::api::Result<()> {
		#[cfg(windows)]
		{
			if crate::utils::platform::is_windows_7() {
				self.notify_win7(app)
			} else {
				#[allow(deprecated)]
				self.show()
			}
		}
		#[cfg(not(windows))]
		{
			#[allow(deprecated)]
			self.show()
		}
	}

	#[cfg(all(windows, feature = "windows7-compat"))]
	fn notify_win7<R: crate::Runtime>(self, app: &crate::AppHandle<R>) -> crate::api::Result<()> {
		let app = app.clone();
		let default_window_icon = app.manager.inner.default_window_icon.clone();
		let _ = app.run_on_main_thread(move || {
			let mut notification = win7_notifications::Notification::new();
			if let Some(body) = self.body {
				notification.body(&body);
			}
			if let Some(title) = self.title {
				notification.summary(&title);
			}
			if let Some(crate::Icon::Rgba { rgba, width, height }) = default_window_icon {
				notification.icon(rgba, width, height);
			}
			let _ = notification.show();
		});

		Ok(())
	}
}
