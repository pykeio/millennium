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

#![cfg(target_os = "linux")]

#[cfg(feature = "tray")]
use std::path::Path;

#[cfg(feature = "tray")]
use crate::system_tray::SystemTrayBuilder;

#[cfg(feature = "tray")]
pub trait SystemTrayBuilderExtLinux {
	/// Sets a custom temp icon dir to store generated icon files.
	fn with_temp_icon_dir<P: AsRef<Path>>(self, p: P) -> Self;
}

#[cfg(feature = "tray")]
impl SystemTrayBuilderExtLinux for SystemTrayBuilder {
	fn with_temp_icon_dir<P: AsRef<Path>>(mut self, p: P) -> Self {
		self.0.temp_icon_dir = Some(p.as_ref().to_path_buf());
		self
	}
}
