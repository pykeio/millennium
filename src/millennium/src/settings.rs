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

//! The Millennium-specific settings for your application.
//!
//! This only contains notification permission status for now, but is able to
//! expand in the future.

use std::{
	fs::File,
	io::Write,
	path::{Path, PathBuf}
};

use serde::{Deserialize, Serialize};

use crate::{
	api::{
		file::read_binary,
		path::{resolve_path, BaseDirectory}
	},
	Config, Env, PackageInfo
};

/// The Millennium Settings.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Settings {
	/// Whether the user allows notifications or not.
	#[cfg(notification_all)]
	pub allow_notification: Option<bool>
}

/// Gets the path to the settings file.
fn get_settings_path(config: &Config, package_info: &PackageInfo, env: &Env) -> crate::api::Result<PathBuf> {
	resolve_path(config, package_info, env, ".millennium-settings", Some(BaseDirectory::App))
}

/// Write the settings to the file system.
#[allow(dead_code)]
pub(crate) fn write_settings(config: &Config, package_info: &PackageInfo, env: &Env, settings: Settings) -> crate::Result<()> {
	let settings_path = get_settings_path(config, package_info, env)?;
	let settings_folder = Path::new(&settings_path).parent().unwrap();
	if !settings_folder.exists() {
		std::fs::create_dir(settings_folder)?;
	}
	File::create(settings_path).map_err(Into::into).and_then(|mut f| {
		f.write_all(&bincode::serialize(&settings).map_err(crate::api::Error::Bincode)?)
			.map_err(Into::into)
	})
}

/// Reads the settings from the file system.
pub fn read_settings(config: &Config, package_info: &PackageInfo, env: &Env) -> Settings {
	if let Ok(settings_path) = get_settings_path(config, package_info, env) {
		if settings_path.exists() {
			read_binary(settings_path)
				.and_then(|settings| bincode::deserialize(&settings).map_err(Into::into))
				.unwrap_or_default()
		} else {
			Settings::default()
		}
	} else {
		Settings::default()
	}
}
