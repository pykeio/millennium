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

use windows::Win32::{Foundation::HWND, UI::Input::KeyboardAndMouse::*};

use super::keyboard::key_to_vk;
use crate::{
	accelerator::{Accelerator, AcceleratorId},
	event_loop::EventLoopWindowTarget,
	global_shortcut::{GlobalShortcut as RootGlobalShortcut, ShortcutManagerError},
	keyboard::ModifiersState
};

#[derive(Debug, Clone)]
pub struct ShortcutManager {
	shortcuts: Vec<GlobalShortcut>
}

impl ShortcutManager {
	pub(crate) fn new<T>(_window_target: &EventLoopWindowTarget<T>) -> Self {
		ShortcutManager { shortcuts: Vec::new() }
	}

	pub(crate) fn register(&mut self, accelerator: Accelerator) -> Result<RootGlobalShortcut, ShortcutManagerError> {
		unsafe {
			let mut converted_modifiers = Default::default();
			let modifiers: ModifiersState = accelerator.mods;
			if modifiers.shift_key() {
				converted_modifiers |= MOD_SHIFT;
			}
			if modifiers.super_key() {
				converted_modifiers |= MOD_WIN;
			}
			if modifiers.alt_key() {
				converted_modifiers |= MOD_ALT;
			}
			if modifiers.control_key() {
				converted_modifiers |= MOD_CONTROL;
			}

			// get key scan code
			match key_to_vk(&accelerator.key) {
				Some(vk_code) => {
					let result = RegisterHotKey(HWND::default(), accelerator.clone().id().0 as i32, converted_modifiers, u32::from(vk_code.0));
					if !result.as_bool() {
						return Err(ShortcutManagerError::InvalidAccelerator("Unable to register accelerator with `RegisterHotKey`.".into()));
					}
					let shortcut = GlobalShortcut { accelerator };
					self.shortcuts.push(shortcut.clone());
					Ok(RootGlobalShortcut(shortcut))
				}
				_ => Err(ShortcutManagerError::InvalidAccelerator("Unable to register accelerator (unknown VKCode for this char).".into()))
			}
		}
	}

	pub(crate) fn unregister_all(&self) -> Result<(), ShortcutManagerError> {
		for shortcut in &self.shortcuts {
			shortcut.unregister();
		}
		Ok(())
	}

	pub(crate) fn unregister(&self, shortcut: RootGlobalShortcut) -> Result<(), ShortcutManagerError> {
		shortcut.0.unregister();
		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalShortcut {
	pub(crate) accelerator: Accelerator
}

impl GlobalShortcut {
	pub fn id(&self) -> AcceleratorId {
		self.accelerator.clone().id()
	}
	pub(crate) fn unregister(&self) {
		unsafe {
			UnregisterHotKey(HWND::default(), self.accelerator.clone().id().0 as i32);
		}
	}
}
