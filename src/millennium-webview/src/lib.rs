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

//! Millennium Webview is a Cross-platform WebView rendering library.
//!
//! To build a Window with WebView embedded, we could use [`application`] module
//! to create [`EventLoop`] and the window. It's a module that re-exports APIs
//! from `millennium-core`. Then use [`webview`] module to create the
//! [`WebView`] from the [`Window`]. Here's a minimum example showing how to
//! create a basic window:
//!
//! ```no_run
//! fn main() -> millennium_webview::Result<()> {
//! 	use millennium_webview::{
//! 		application::{
//! 			event::{Event, StartCause, WindowEvent},
//! 			event_loop::{ControlFlow, EventLoop},
//! 			window::WindowBuilder
//! 		},
//! 		webview::WebViewBuilder
//! 	};
//!
//! 	let event_loop = EventLoop::new();
//! 	let window = WindowBuilder::new().with_title("Hello World").build(&event_loop)?;
//! 	let _webview = WebViewBuilder::new(window)?.with_url("https://pyke.io/")?.build()?;
//!
//! 	event_loop.run(move |event, _, control_flow| {
//! 		*control_flow = ControlFlow::Wait;
//!
//! 		match event {
//! 			Event::NewEvents(StartCause::Init) => println!("Millennium Webview has started!"),
//! 			Event::WindowEvent {
//! 				event: WindowEvent::CloseRequested, ..
//! 			} => *control_flow = ControlFlow::Exit,
//! 			_ => ()
//! 		}
//! 	});
//! }
//! ```
//!
//! ## Feature flags
//!
//! Millennium Webview uses a set of feature flags to toggle several advanced
//! features. `file-drop`, `protocol`, and `tray` are enabled by default.
//!
//! - `file-drop`: Enables [`with_file_drop_handler`] to control the behaviour when there are files
//! interacting with the window. Enabled by default.
//! - `protocol`: Enables [`with_custom_protocol`] to define custom URL scheme for handling tasks like
//! loading assets. Enabled by default.
//! - `tray`: Enables system tray and more menu item variants on **Linux**. You can still create
//! those types if you disable it. They just don't create the actual objects. We
//! set this flag because some implementations require more installed packages.
//! Disable this if you don't want to install `libappindicator` package. Enabled
//! by default.
//! - `ayatana`: Enable this if you wish to use more update `libayatana-appindicator` since
//! `libappindicator` is no longer maintained.
//! - `devtools`: Enables devtools in release builds. Devtools are always enabled in debug builds.
//! On macOS, enabling devtools requires calling private functions, so you should avoid using this in release builds if
//! your app needs to be published to the App Store.
//! - `transparent`: Transparent background on **macOS** requires calling private functions.
//! Avoid this in release build if your app needs to publish to App Store.
//! - `fullscreen`: Fullscreen video and other media on **macOS** requires calling private functions.
//! Avoid this in release build if your app needs to publish to App Store.
//! - `dox`: Enables this in `package.metadata.docs.rs` section to skip linking some **Linux**
//! libraries and prevent from building documentation on doc.rs fails.
//!
//! [`EventLoop`]: crate::application::event_loop::EventLoop
//! [`Window`]: crate::application::window::Window
//! [`WebView`]: crate::webview::WebView
//! [`with_file_drop_handler`]: crate::webview::WebView::with_file_drop_handler
//! [`with_custom_protocol`]: crate::webview::WebView::with_custom_protocol

#![allow(clippy::new_without_default)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::type_complexity)]
#![allow(clippy::unit_cmp)]
#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;
#[cfg(any(target_os = "macos", target_os = "ios"))]
#[macro_use]
extern crate objc;

use std::sync::mpsc::{RecvError, SendError};

pub use serde_json::Value;
use url::ParseError;

use crate::{
	application::window::BadIcon,
	http::{
		header::{InvalidHeaderName, InvalidHeaderValue},
		method::InvalidMethod,
		status::InvalidStatusCode,
		InvalidUri
	}
};

pub mod application;
pub mod http;
pub mod webview;

/// Convenient type alias of Result type for Millennium Webview.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by Millennium Webview.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
	#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
	#[error(transparent)]
	GlibError(#[from] glib::Error),
	#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
	#[error(transparent)]
	GlibBoolError(#[from] glib::BoolError),
	#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
	#[error("Fail to fetch security manager")]
	MissingManager,
	#[error("Failed to initialize the script")]
	InitScriptError,
	#[error("Bad RPC request: {0} ((1))")]
	RpcScriptError(String, String),
	#[error(transparent)]
	NulError(#[from] std::ffi::NulError),
	#[error(transparent)]
	OsError(#[from] crate::application::error::OsError),
	#[error(transparent)]
	ReceiverError(#[from] RecvError),
	#[error(transparent)]
	SenderError(#[from] SendError<String>),
	#[error("Failed to send the message")]
	MessageSender,
	#[error(transparent)]
	Json(#[from] serde_json::Error),
	#[error(transparent)]
	UrlError(#[from] ParseError),
	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),
	#[error("Icon error: {0}")]
	Icon(#[from] BadIcon),
	#[cfg(target_os = "windows")]
	#[error("WebView2 error: {0}")]
	WebView2Error(webview2_com::Error),
	#[error("Duplicate custom protocol registered: {0}")]
	DuplicateCustomProtocol(String),
	#[error("Invalid header name: {0}")]
	InvalidHeaderName(#[from] InvalidHeaderName),
	#[error("Invalid header value: {0}")]
	InvalidHeaderValue(#[from] InvalidHeaderValue),
	#[error("Invalid uri: {0}")]
	InvalidUri(#[from] InvalidUri),
	#[error("Invalid status code: {0}")]
	InvalidStatusCode(#[from] InvalidStatusCode),
	#[error("Invalid method: {0}")]
	InvalidMethod(#[from] InvalidMethod),
	#[error("Infallible error, something went really wrong: {0}")]
	Infallible(#[from] std::convert::Infallible),
	#[cfg(target_os = "android")]
	#[error("JNI error: {0}")]
	JNIError(#[from] jni::errors::Error)
}
