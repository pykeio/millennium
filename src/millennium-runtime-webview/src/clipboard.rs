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

//! Clipboard implementation.

use std::sync::{
	mpsc::{channel, Sender},
	Arc, Mutex
};

use millennium_runtime::{ClipboardManager, Result, UserEvent};
pub use millennium_webview::application::clipboard::Clipboard;

use crate::{getter, Context, Message};

#[derive(Debug, Clone)]
pub enum ClipboardMessage {
	WriteText(String, Sender<()>),
	ReadText(Sender<Option<String>>)
}

#[derive(Debug, Clone)]
pub struct ClipboardManagerWrapper<T: UserEvent> {
	pub context: Context<T>
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for ClipboardManagerWrapper<T> {}

impl<T: UserEvent> ClipboardManager for ClipboardManagerWrapper<T> {
	fn read_text(&self) -> Result<Option<String>> {
		let (tx, rx) = channel();
		getter!(self, rx, Message::Clipboard(ClipboardMessage::ReadText(tx)))
	}

	fn write_text<V: Into<String>>(&mut self, text: V) -> Result<()> {
		let (tx, rx) = channel();
		getter!(self, rx, Message::Clipboard(ClipboardMessage::WriteText(text.into(), tx)))?;
		Ok(())
	}
}

pub fn handle_clipboard_message(message: ClipboardMessage, clipboard_manager: &Arc<Mutex<Clipboard>>) {
	match message {
		ClipboardMessage::WriteText(text, tx) => {
			clipboard_manager.lock().unwrap().write_text(text);
			tx.send(()).unwrap();
		}
		ClipboardMessage::ReadText(tx) => tx.send(clipboard_manager.lock().unwrap().read_text()).unwrap()
	}
}
