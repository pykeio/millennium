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

//! Make your Millennium windows vibrant.
//!
//! # Platform support
//!
//! - **Windows:** Yes!
//! - **macOS:** Yes!
//! - **Linux:** No, blur effects are controlled by the compositor and they can enable it for your app if they want.
//!
//! # Example with Millennium Core:
//!
//! ```no_run
//! # use millennium_core::{event_loop::EventLoop, window::WindowBuilder};
//! # use millennium_plugin_vibrancy::{apply_vibrancy, apply_blur, NSVisualEffectMaterial};
//! let event_loop = EventLoop::new();
//!
//! let window = WindowBuilder::new().with_decorations(false).build(&event_loop).unwrap();
//!
//! #[cfg(target_os = "windows")]
//! apply_blur(&window, Some((18, 18, 18, 125))).unwrap();
//!
//! #[cfg(target_os = "macos")]
//! apply_vibrancy(&window, NSVisualEffectMaterial::AppearanceBased).unwrap();
//! ```

mod macos;
mod windows;

pub use macos::NSVisualEffectMaterial;
#[cfg(windows)]
pub use windows::{is_win10_swca as is_win10, is_win11, is_win7};

/// a tuple of RGBA colors. Each value has a range of 0 to 255.
pub type Color = (u8, u8, u8, u8);

/// Applies blur effect to the window. Works only on Windows 7, Windows 10 v1809
/// or newer, and Windows 11.
///
/// - *`color`* is ignored on Windows 7 and has no effect.
/// - This may be laggy when the window is resized or moved on some Windows 11 installs. For recent Windows versions,
///   use `apply_acrylic` or `apply_mica` instead.
pub fn apply_blur(window: impl raw_window_handle::HasRawWindowHandle, #[allow(unused)] color: Option<Color>) -> Result<(), Error> {
	match window.raw_window_handle() {
		#[cfg(target_os = "windows")]
		raw_window_handle::RawWindowHandle::Win32(handle) => windows::apply_blur(handle.hwnd as _, color),
		_ => Err(Error::UnsupportedPlatform("\"apply_blur()\" is only supported on Windows."))
	}
}

/// Clears blur effect applied to the window. Works only on Windows 7, Windows
/// 10 v1809 or newer, and Windows 11.
pub fn clear_blur(window: impl raw_window_handle::HasRawWindowHandle) -> Result<(), Error> {
	match window.raw_window_handle() {
		#[cfg(target_os = "windows")]
		raw_window_handle::RawWindowHandle::Win32(handle) => windows::clear_blur(handle.hwnd as _),
		_ => Err(Error::UnsupportedPlatform("\"clear_blur()\" is only supported on Windows."))
	}
}

/// Applies Acrylic effect to the window. Works only on Windows 10 v1809 or
/// newer and Windows 11.
///
/// - *`color`* is ignored on Windows 11 build 22523 and newer and has no effect. Instead, you should set the background
///   color of the webview to some transparent color if you want to tint the window.
/// - This may also be laggy on Windows 10 v1903+ and Windows 11 builds prior to build 22523, the window may lag when
///   resizing or dragging.
pub fn apply_acrylic(window: impl raw_window_handle::HasRawWindowHandle, #[allow(unused)] color: Option<Color>) -> Result<(), Error> {
	match window.raw_window_handle() {
		#[cfg(target_os = "windows")]
		raw_window_handle::RawWindowHandle::Win32(handle) => windows::apply_acrylic(handle.hwnd as _, color),
		_ => Err(Error::UnsupportedPlatform("\"apply_acrylic()\" is only supported on Windows."))
	}
}

/// Clears Acrylic effect applied to the window. Works only on Windows 10 v1809
/// or newer and Windows 11.
pub fn clear_acrylic(window: impl raw_window_handle::HasRawWindowHandle) -> Result<(), Error> {
	match window.raw_window_handle() {
		#[cfg(target_os = "windows")]
		raw_window_handle::RawWindowHandle::Win32(handle) => windows::clear_acrylic(handle.hwnd as _),
		_ => Err(Error::UnsupportedPlatform("\"clear_acrylic()\" is only supported on Windows."))
	}
}

/// Applies Mica effect to the window. Works only on Windows 11.
///
/// - *`color`* is not supported, though you shouldn't have to worry about tinting; the window will be quite dark if a
///   dark system theme is enabled and very light if a light theme is enabled.
pub fn apply_mica(window: impl raw_window_handle::HasRawWindowHandle) -> Result<(), Error> {
	match window.raw_window_handle() {
		#[cfg(target_os = "windows")]
		raw_window_handle::RawWindowHandle::Win32(handle) => windows::apply_mica(handle.hwnd as _),
		_ => Err(Error::UnsupportedPlatform("\"apply_mica()\" is only supported on Windows."))
	}
}

/// Clears Mica effect applied to the window. Works only on Windows 10 v1903
/// or newer and Windows 11.
pub fn clear_mica(window: impl raw_window_handle::HasRawWindowHandle) -> Result<(), Error> {
	match window.raw_window_handle() {
		#[cfg(target_os = "windows")]
		raw_window_handle::RawWindowHandle::Win32(handle) => windows::clear_mica(handle.hwnd as _),
		_ => Err(Error::UnsupportedPlatform("\"clear_mica()\" is only supported on Windows."))
	}
}

/// Applies macOS Vibrancy effect to the window. Works only on macOS 10.10 or
/// newer.
pub fn apply_vibrancy(window: impl raw_window_handle::HasRawWindowHandle, #[allow(unused)] effect: NSVisualEffectMaterial) -> Result<(), Error> {
	match window.raw_window_handle() {
		#[cfg(target_os = "macos")]
		raw_window_handle::RawWindowHandle::AppKit(handle) => macos::apply_vibrancy(handle.ns_window as _, effect),
		_ => Err(Error::UnsupportedPlatform("\"apply_vibrancy()\" is only supported on macOS."))
	}
}

#[derive(Debug)]
pub enum Error {
	UnsupportedPlatform(&'static str),
	UnsupportedPlatformVersion(&'static str),
	NotMainThread(&'static str)
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::UnsupportedPlatform(e) | Error::UnsupportedPlatformVersion(e) | Error::NotMainThread(e) => {
				write!(f, "{}", e)
			}
		}
	}
}
