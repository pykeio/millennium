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

use cocoa::{
	appkit::NSPasteboardTypeString,
	base::{id, nil, BOOL, YES},
	foundation::{NSInteger, NSString}
};
use objc::{class, msg_send, sel, sel_impl};

#[derive(Debug, Clone, Default)]
pub struct Clipboard;

impl Clipboard {
	pub(crate) fn write_text(&mut self, s: impl AsRef<str>) {
		let s = s.as_ref();
		unsafe {
			let nsstring = NSString::alloc(nil).init_str(s);
			let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
			let _: NSInteger = msg_send![pasteboard, clearContents];
			let result: BOOL = msg_send![pasteboard, setString: nsstring forType: NSPasteboardTypeString];
			if result != YES {
				#[cfg(debug_assertions)]
				println!("failed to set clipboard");
			}
		}
	}

	pub(crate) fn read_text(&self) -> Option<String> {
		unsafe {
			let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
			let contents: id = msg_send![pasteboard, stringForType: NSPasteboardTypeString];
			if contents.is_null() {
				None
			} else {
				let slice = std::slice::from_raw_parts(contents.UTF8String() as *const _, contents.len());
				let result = std::str::from_utf8_unchecked(slice);
				Some(result.to_string())
			}
		}
	}
}
