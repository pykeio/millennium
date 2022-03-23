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

use colored::Colorize;

#[derive(Clone)]
pub struct Logger<'a> {
	context: &'a str
}

impl<'a> Logger<'a> {
	pub fn new(context: &'a str) -> Self {
		Self { context }
	}

	pub fn log(&self, message: impl AsRef<str>) {
		println!("{} {}", format!("[{}]", self.context).green().bold(), message.as_ref());
	}

	pub fn warn(&self, message: impl AsRef<str>) {
		println!("{} {}", format!("[{}]", self.context).yellow().bold(), message.as_ref());
	}

	#[allow(dead_code)]
	pub fn error(&self, message: impl AsRef<str>) {
		println!("{} {}", format!("[{}]", self.context).red().bold(), message.as_ref());
	}
}
