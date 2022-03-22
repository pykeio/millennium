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

pub mod app_paths;
pub mod config;
pub mod framework;
mod logger;
pub mod manifest;
pub mod template;
pub mod updater_signature;

use std::{
	collections::HashMap,
	path::{Path, PathBuf}
};

pub use logger::Logger;

pub fn command_env(debug: bool) -> HashMap<String, String> {
	let mut map = HashMap::new();

	map.insert("MILLENNIUM_PLATFORM".into(), std::env::consts::OS.into());
	map.insert("MILLENNIUM_ARCH".into(), std::env::consts::ARCH.into());
	map.insert("MILLENNIUM_FAMILY".into(), std::env::consts::FAMILY.into());
	map.insert("MILLENNIUM_PLATFORM_VERSION".into(), os_info::get().version().to_string());

	#[cfg(target_os = "linux")]
	map.insert("MILLENNIUM_PLATFORM_TYPE".into(), "Linux".into());
	#[cfg(target_os = "windows")]
	map.insert("MILLENNIUM_PLATFORM_TYPE".into(), "Windows_NT".into());
	#[cfg(target_os = "macos")]
	map.insert("MILLENNIUM_PLATFORM_TYPE".into(), "Darwin".into());

	if debug {
		map.insert("MILLENNIUM_DEBUG".into(), "true".to_string());
	}

	map
}

pub fn resolve_millennium_path<P: AsRef<Path>>(path: P, crate_name: &str) -> PathBuf {
	let path = path.as_ref();
	if path.is_absolute() {
		path.join(crate_name)
	} else {
		PathBuf::from("..").join(path).join(crate_name)
	}
}
