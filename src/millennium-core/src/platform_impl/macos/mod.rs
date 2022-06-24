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

#![cfg(target_os = "macos")]

mod app;
mod app_delegate;
mod app_state;
mod clipboard;
mod event;
mod event_loop;
mod ffi;
mod global_shortcut;
mod icon;
mod keycode;
mod menu;
mod monitor;
mod observer;
#[cfg(feature = "tray")]
mod system_tray;
mod util;
mod view;
mod window;
mod window_delegate;

use std::{fmt, ops::Deref, sync::Arc};

pub(crate) use icon::PlatformIcon;

#[cfg(feature = "tray")]
pub use self::system_tray::{SystemTray, SystemTrayBuilder};
pub use self::{
	app_delegate::{get_aux_state_mut, AuxDelegateState},
	clipboard::Clipboard,
	event::KeyEventExtra,
	event_loop::{EventLoop, EventLoopWindowTarget, Proxy as EventLoopProxy},
	global_shortcut::{GlobalShortcut, ShortcutManager},
	keycode::{keycode_from_scancode, keycode_to_scancode},
	menu::{Menu, MenuItemAttributes},
	monitor::{MonitorHandle, VideoMode},
	window::{Id as WindowId, Parent, PlatformSpecificWindowBuilderAttributes, UnownedWindow}
};
use crate::{error::OsError as RootOsError, event::DeviceId as RootDeviceId, window::WindowAttributes};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId;

impl DeviceId {
	pub unsafe fn dummy() -> Self {
		DeviceId
	}
}

// Constant device ID; to be removed when if backend is updated to report real
// device IDs.
pub(crate) const DEVICE_ID: RootDeviceId = RootDeviceId(DeviceId);

pub struct Window {
	window: Arc<UnownedWindow>,
	// We keep this around so that it doesn't get dropped until the window does.
	delegate: util::IdRef
}

#[non_exhaustive]
#[derive(Debug)]
pub enum OsError {
	CGError(core_graphics::base::CGError),
	CreationError(&'static str)
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl Deref for Window {
	type Target = UnownedWindow;
	#[inline]
	fn deref(&self) -> &Self::Target {
		&*self.window
	}
}

impl Window {
	pub fn new<T: 'static>(
		_window_target: &EventLoopWindowTarget<T>,
		attributes: WindowAttributes,
		pl_attribs: PlatformSpecificWindowBuilderAttributes
	) -> Result<Self, RootOsError> {
		let (window, delegate) = UnownedWindow::new(attributes, pl_attribs)?;
		Ok(Window { window, delegate })
	}

	#[inline]
	pub fn is_maximized(&self) -> bool {
		let () = unsafe { msg_send![*self.delegate, markIsCheckingZoomedIn] };
		let f = self.window.is_zoomed();
		let () = unsafe { msg_send![*self.delegate, clearIsCheckingZoomedIn] };
		f
	}
}

impl fmt::Display for OsError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			OsError::CGError(e) => f.pad(&format!("CGError {}", e)),
			OsError::CreationError(e) => f.pad(e)
		}
	}
}
