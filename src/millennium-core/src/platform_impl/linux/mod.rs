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

#![cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]

mod clipboard;
mod event_loop;
mod global_shortcut;
mod keyboard;
mod keycode;
mod menu;
mod monitor;
#[cfg(feature = "tray")]
mod system_tray;
mod window;

pub use event_loop::{EventLoop, EventLoopProxy, EventLoopWindowTarget};
pub use monitor::{MonitorHandle, VideoMode};
pub use window::{hit_test, PlatformIcon, Window, WindowId};

#[cfg(feature = "tray")]
pub use self::system_tray::{SystemTray, SystemTrayBuilder};
pub use self::{
	clipboard::Clipboard,
	global_shortcut::{GlobalShortcut, ShortcutManager},
	keycode::{keycode_from_scancode, keycode_to_scancode},
	menu::{Menu, MenuItemAttributes}
};
use crate::{event::DeviceId as RootDeviceId, keyboard::Key};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KeyEventExtra {
	pub text_with_all_modifiers: Option<&'static str>,
	pub key_without_modifiers: Key<'static>
}

#[derive(Clone, Default)]
pub struct PlatformSpecificWindowBuilderAttributes {
	pub skip_taskbar: bool
}

unsafe impl Send for PlatformSpecificWindowBuilderAttributes {}
unsafe impl Sync for PlatformSpecificWindowBuilderAttributes {}

#[derive(Debug, Clone)]
pub struct OsError;

impl std::fmt::Display for OsError {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		Ok(())
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(usize);

impl DeviceId {
	pub unsafe fn dummy() -> Self {
		Self(0)
	}
}

// FIXME: currently we use a dummy device id, find if we can get device id from
// gtk
pub(crate) const DEVICE_ID: RootDeviceId = RootDeviceId(DeviceId(0));
