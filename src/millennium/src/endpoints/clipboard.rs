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

use millennium_macros::{module_command_handler, CommandModule};
use serde::Deserialize;

use super::InvokeContext;
#[cfg(any(clipboard_write_text, clipboard_read_text))]
use crate::runtime::ClipboardManager;
use crate::Runtime;

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", content = "data", rename_all = "camelCase")]
pub enum Cmd {
	/// Write a text string to the clipboard.
	WriteText(String),
	/// Read clipboard content as text.
	ReadText
}

impl Cmd {
	#[module_command_handler(clipboard_write_text, "clipboard > writeText")]
	fn write_text<R: Runtime>(context: InvokeContext<R>, text: String) -> super::Result<()> {
		context.window.app_handle.clipboard_manager().write_text(text).map_err(crate::error::into_anyhow)
	}

	#[module_command_handler(clipboard_read_text, "clipboard > readText")]
	fn read_text<R: Runtime>(context: InvokeContext<R>) -> super::Result<Option<String>> {
		context.window.app_handle.clipboard_manager().read_text().map_err(crate::error::into_anyhow)
	}
}

#[cfg(test)]
mod tests {
	#[millennium_macros::module_command_test(clipboard_write_text, "clipboard > writeText")]
	#[quickcheck_macros::quickcheck]
	fn write_text(text: String) {
		let ctx = crate::test::mock_invoke_context();
		super::Cmd::write_text(ctx.clone(), text.clone()).unwrap();
		assert_eq!(super::Cmd::read_text(ctx).unwrap(), Some(text));
	}

	#[millennium_macros::module_command_test(clipboard_read_text, "clipboard > readText")]
	#[quickcheck_macros::quickcheck]
	fn read_text() {
		let ctx = crate::test::mock_invoke_context();
		assert_eq!(super::Cmd::read_text(ctx.clone()).unwrap(), None);
		let text = "Millennium!".to_string();
		super::Cmd::write_text(ctx.clone(), text.clone()).unwrap();
		assert_eq!(super::Cmd::read_text(ctx).unwrap(), Some(text));
	}
}
