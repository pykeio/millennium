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

pub mod rust;

use std::path::Path;

use millennium_bundler::bundle::{PackageType, Settings, SettingsBuilder};

use crate::helpers::{config::Config, manifest::Manifest};

pub fn get_bundler_settings(
	app_settings: rust::AppSettings,
	target: Option<String>,
	manifest: &Manifest,
	config: &Config,
	out_dir: &Path,
	verbose: bool,
	package_types: Option<Vec<PackageType>>
) -> crate::Result<Settings> {
	let mut settings_builder = SettingsBuilder::new()
		.package_settings(app_settings.get_package_settings())
		.bundle_settings(app_settings.get_bundle_settings(config, manifest)?)
		.binaries(app_settings.get_binaries(config)?)
		.project_out_directory(out_dir);

	if verbose {
		settings_builder = settings_builder.verbose();
	}

	if let Some(types) = package_types {
		settings_builder = settings_builder.package_types(types);
	}

	if let Some(target) = target {
		settings_builder = settings_builder.target(target);
	}

	settings_builder.build().map_err(Into::into)
}
