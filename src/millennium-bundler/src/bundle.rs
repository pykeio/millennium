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

mod category;
mod common;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod path_utils;
mod platform;
mod settings;
mod updater_bundle;
#[cfg(target_os = "windows")]
mod windows;

use std::path::PathBuf;

use common::{print_finished, print_info};
pub use settings::{WindowsSettings, WixLanguage, WixLanguageConfig, WixSettings};

pub use self::{
	category::AppCategory,
	settings::{BundleBinary, BundleSettings, DebianSettings, MacOsSettings, PackageSettings, PackageType, Settings, SettingsBuilder, UpdaterSettings}
};

/// Generated bundle metadata.
#[derive(Debug)]
pub struct Bundle {
	/// The package type.
	pub package_type: PackageType,
	/// All paths for this package.
	pub bundle_paths: Vec<PathBuf>
}

/// Bundles the project.
/// Returns the list of paths where the bundles can be found.
pub fn bundle_project(settings: Settings) -> crate::Result<Vec<Bundle>> {
	let mut bundles = Vec::new();
	let package_types = settings.package_types()?;

	for package_type in &package_types {
		let bundle_paths = match package_type {
			#[cfg(target_os = "macos")]
			PackageType::MacOsBundle => macos::app::bundle_project(&settings)?,
			#[cfg(target_os = "macos")]
			PackageType::IosBundle => macos::ios::bundle_project(&settings)?,
			#[cfg(target_os = "windows")]
			PackageType::WindowsMsi => windows::msi::bundle_project(&settings)?,
			#[cfg(target_os = "linux")]
			PackageType::Deb => linux::debian::bundle_project(&settings)?,
			#[cfg(target_os = "linux")]
			PackageType::Rpm => linux::rpm::bundle_project(&settings)?,
			#[cfg(target_os = "linux")]
			PackageType::AppImage => linux::appimage::bundle_project(&settings)?,
			// dmg is dependant of MacOsBundle, we send our bundles to prevent rebuilding
			#[cfg(target_os = "macos")]
			PackageType::Dmg => macos::dmg::bundle_project(&settings, &bundles)?,
			// updater is dependant of multiple bundle, we send our bundles to prevent rebuilding
			PackageType::Updater => updater_bundle::bundle_project(&settings, &bundles)?,
			_ => {
				print_info(&format!("ignoring {:?}", package_type))?;
				continue;
			}
		};

		bundles.push(Bundle {
			package_type: package_type.to_owned(),
			bundle_paths
		});
	}

	print_finished(&bundles)?;

	Ok(bundles)
}

/// Check to see if there are icons in the settings struct
pub fn check_icons(settings: &Settings) -> crate::Result<bool> {
	// make a peekable iterator of the icon_files
	let mut iter = settings.icon_files().peekable();

	// if iter's first value is a None then there are no Icon files in the settings struct
	if iter.peek().is_none() { Ok(false) } else { Ok(true) }
}
