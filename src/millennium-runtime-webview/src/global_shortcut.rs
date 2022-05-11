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

//! Global shortcut implementation.

use std::{
	collections::HashMap,
	fmt,
	sync::{
		mpsc::{channel, Sender},
		Arc, Mutex
	}
};

use millennium_runtime::{Error, GlobalShortcutManager, Result, UserEvent};
pub use millennium_webview::application::global_shortcut::ShortcutManager as MillenniumShortcutManager;
use millennium_webview::application::{
	accelerator::{Accelerator, AcceleratorId},
	global_shortcut::GlobalShortcut
};

use crate::{getter, Context, Message};

pub type GlobalShortcutListeners = Arc<Mutex<HashMap<AcceleratorId, Box<dyn Fn() + Send>>>>;

#[derive(Debug, Clone)]
pub enum GlobalShortcutMessage {
	IsRegistered(Accelerator, Sender<bool>),
	Register(Accelerator, Sender<Result<GlobalShortcutWrapper>>),
	Unregister(GlobalShortcutWrapper, Sender<Result<()>>),
	UnregisterAll(Sender<Result<()>>)
}

#[derive(Debug, Clone)]
pub struct GlobalShortcutWrapper(GlobalShortcut);

// SAFETY: usage outside of main thread is guarded, we use the event loop on such cases.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GlobalShortcutWrapper {}

/// Wrapper around [`WryShortcutManager`].
#[derive(Clone)]
pub struct GlobalShortcutManagerHandle<T: UserEvent> {
	pub context: Context<T>,
	pub shortcuts: Arc<Mutex<HashMap<String, (AcceleratorId, GlobalShortcutWrapper)>>>,
	pub listeners: GlobalShortcutListeners
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for GlobalShortcutManagerHandle<T> {}

impl<T: UserEvent> fmt::Debug for GlobalShortcutManagerHandle<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("GlobalShortcutManagerHandle")
			.field("context", &self.context)
			.field("shortcuts", &self.shortcuts)
			.finish()
	}
}

impl<T: UserEvent> GlobalShortcutManager for GlobalShortcutManagerHandle<T> {
	fn is_registered(&self, accelerator: &str) -> Result<bool> {
		let (tx, rx) = channel();
		getter!(self, rx, Message::GlobalShortcut(GlobalShortcutMessage::IsRegistered(accelerator.parse().expect("invalid accelerator"), tx)))
	}

	fn register<F: Fn() + Send + 'static>(&mut self, accelerator: &str, handler: F) -> Result<()> {
		let wry_accelerator: Accelerator = accelerator.parse().expect("invalid accelerator");
		let id = wry_accelerator.clone().id();
		let (tx, rx) = channel();
		let shortcut = getter!(self, rx, Message::GlobalShortcut(GlobalShortcutMessage::Register(wry_accelerator, tx)))??;

		self.listeners.lock().unwrap().insert(id, Box::new(handler));
		self.shortcuts.lock().unwrap().insert(accelerator.into(), (id, shortcut));

		Ok(())
	}

	fn unregister_all(&mut self) -> Result<()> {
		let (tx, rx) = channel();
		getter!(self, rx, Message::GlobalShortcut(GlobalShortcutMessage::UnregisterAll(tx)))??;
		self.listeners.lock().unwrap().clear();
		self.shortcuts.lock().unwrap().clear();
		Ok(())
	}

	fn unregister(&mut self, accelerator: &str) -> Result<()> {
		if let Some((accelerator_id, shortcut)) = self.shortcuts.lock().unwrap().remove(accelerator) {
			let (tx, rx) = channel();
			getter!(self, rx, Message::GlobalShortcut(GlobalShortcutMessage::Unregister(shortcut, tx)))??;
			self.listeners.lock().unwrap().remove(&accelerator_id);
		}
		Ok(())
	}
}

pub fn handle_global_shortcut_message(message: GlobalShortcutMessage, global_shortcut_manager: &Arc<Mutex<MillenniumShortcutManager>>) {
	match message {
		GlobalShortcutMessage::IsRegistered(accelerator, tx) => tx.send(global_shortcut_manager.lock().unwrap().is_registered(&accelerator)).unwrap(),
		GlobalShortcutMessage::Register(accelerator, tx) => tx
			.send(
				global_shortcut_manager
					.lock()
					.unwrap()
					.register(accelerator)
					.map(GlobalShortcutWrapper)
					.map_err(|e| Error::GlobalShortcut(Box::new(e)))
			)
			.unwrap(),
		GlobalShortcutMessage::Unregister(shortcut, tx) => tx
			.send(
				global_shortcut_manager
					.lock()
					.unwrap()
					.unregister(shortcut.0)
					.map_err(|e| Error::GlobalShortcut(Box::new(e)))
			)
			.unwrap(),
		GlobalShortcutMessage::UnregisterAll(tx) => tx
			.send(
				global_shortcut_manager
					.lock()
					.unwrap()
					.unregister_all()
					.map_err(|e| Error::GlobalShortcut(Box::new(e)))
			)
			.unwrap()
	}
}
