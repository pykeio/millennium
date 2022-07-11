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

use std::{
	path::{Path, PathBuf},
	process::ExitStatus
};

use millennium_bundler::bundle::{PackageType, Settings, SettingsBuilder};
pub use rust::{Options, Rust as AppInterface};

use crate::helpers::config::Config;

pub trait AppSettings {
	fn get_package_settings(&self) -> millennium_bundler::PackageSettings;
	fn get_bundle_settings(&self, config: &Config, features: &[String]) -> crate::Result<millennium_bundler::BundleSettings>;
	fn app_binary_path(&self, options: &Options) -> crate::Result<PathBuf>;
	fn get_binaries(&self, config: &Config, target: &str) -> crate::Result<Vec<millennium_bundler::BundleBinary>>;

	fn get_bundler_settings(&self, options: &Options, config: &Config, out_dir: &Path, package_types: Option<Vec<PackageType>>) -> crate::Result<Settings> {
		let no_default_features = options.args.contains(&"--no-default-features".into());
		let mut enabled_features = options.features.clone().unwrap_or_default();
		if !no_default_features {
			enabled_features.push("default".into());
		}

		let target: String = if let Some(target) = options.target.clone() {
			target
		} else {
			millennium_utils::platform::target_triple()?
		};

		let mut settings_builder = SettingsBuilder::new()
			.package_settings(self.get_package_settings())
			.bundle_settings(self.get_bundle_settings(config, &enabled_features)?)
			.binaries(self.get_binaries(config, &target)?)
			.project_out_directory(out_dir)
			.target(target);

		if let Some(types) = package_types {
			settings_builder = settings_builder.package_types(types);
		}

		settings_builder.build().map_err(Into::into)
	}
}

#[derive(Debug)]
pub enum ExitReason {
	/// Killed manually.
	TriggeredKill,
	/// App compilation failed.
	CompilationFailed,
	/// Regular exit.
	NormalExit
}

pub trait Interface: Sized {
	type AppSettings: AppSettings;

	fn new(config: &Config) -> crate::Result<Self>;
	fn app_settings(&self) -> &Self::AppSettings;
	fn build(&mut self, options: Options) -> crate::Result<()>;
	fn dev<F: Fn(ExitStatus, ExitReason) + Send + Sync + 'static>(&mut self, options: Options, on_exit: F) -> crate::Result<()>;
}
