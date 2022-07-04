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

mod wix;

use std::{self, path::PathBuf};

use log::warn;
pub use wix::{MSI_FOLDER_NAME, MSI_UPDATER_FOLDER_NAME};

use crate::Settings;

const WIX_REQUIRED_FILES: &[&str] = &[
	"candle.exe",
	"candle.exe.config",
	"darice.cub",
	"light.exe",
	"light.exe.config",
	"wconsole.dll",
	"winterop.dll",
	"wix.dll",
	"WixUIExtension.dll",
	"WixUtilExtension.dll"
];

/// Runs all of the commands to build the MSI installer.
/// Returns a vector of PathBuf that shows where the MSI was created.
pub fn bundle_project(settings: &Settings, updater: bool) -> crate::Result<Vec<PathBuf>> {
	let mut wix_path = dirs_next::cache_dir().unwrap();
	wix_path.push("millennium/WixTools");

	if !wix_path.exists() {
		wix::get_and_extract_wix(&wix_path)?;
	} else if WIX_REQUIRED_FILES.iter().any(|p| !wix_path.join(p).exists()) {
		warn!("WixTools directory is missing some files; recreating it");
		std::fs::remove_dir_all(&wix_path)?;
		wix::get_and_extract_wix(&wix_path)?;
	}

	wix::build_wix_app_installer(settings, &wix_path, updater)
}
