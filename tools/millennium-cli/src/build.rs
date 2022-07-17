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

use std::{env::set_current_dir, path::PathBuf, process::Command};

use anyhow::{bail, Context};
use clap::Parser;
use log::{error, info, warn};
use millennium_bundler::bundle::{bundle_project, PackageType};

use crate::{
	helpers::{
		app_paths::{app_dir, millennium_dir},
		command_env,
		config::{get as get_config, AppUrl, WindowUrl, MERGE_CONFIG_EXTENSION_NAME},
		updater_signature::sign_file_from_env_variables
	},
	interface::{AppInterface, AppSettings, Interface},
	CommandExt, Result
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Bundle your Millennium application")]
pub struct Options {
	/// Binary to use to build the application, defaults to `cargo`
	#[clap(short, long)]
	pub runner: Option<String>,
	/// Builds with debug information. By default, `millennium build` performs a release build.
	#[clap(short, long)]
	pub debug: bool,
	/// Target triple to build against.
	///
	/// Must be one of the values outputted by `$rustc --print target-list` or `universal-apple-darwin` for a
	/// universal macOS application. Note that compiling a universal macOS application requires both the
	/// `aarch64-apple-darwin` and `x86_64-apple-darwin` toolchains to be installed.
	#[clap(short, long)]
	pub target: Option<String>,
	/// Space or comma-separated list of Cargo features to activate.
	#[clap(short, long, multiple_occurrences(true), multiple_values(true))]
	pub features: Option<Vec<String>>,
	/// Space or comma-separated list of bundles to package.
	///
	/// Bundles must be one of `deb`, `appimage`, `msi`, `app`, `dmg`, or `updater`.
	///
	/// Note that the `updater` bundle is not automatically added, so you must specify it if the updater is enabled.
	#[clap(short, long, multiple_occurrences(true), multiple_values(true))]
	pub bundles: Option<Vec<String>>,
	/// JSON string or path to JSON file to merge with .millenniumrc
	#[clap(short, long)]
	pub config: Option<String>,
	/// Command line arguments passed to the runner
	pub args: Vec<String>
}

pub fn command(mut options: Options) -> Result<()> {
	let (merge_config, merge_config_path) = if let Some(config) = &options.config {
		if config.starts_with('{') {
			(Some(config.to_string()), None)
		} else {
			(Some(std::fs::read_to_string(&config).with_context(|| "failed to read custom configuration")?), Some(config.clone()))
		}
	} else {
		(None, None)
	};
	options.config = merge_config;

	let millennium_path = millennium_dir();
	set_current_dir(&millennium_path).with_context(|| "failed to change current working directory")?;

	let config = get_config(options.config.as_deref())?;

	let config_guard = config.lock().unwrap();
	let config_ = config_guard.as_ref().unwrap();

	let bundle_identifier_source = match config_.find_bundle_identifier_override() {
		Some(source) if source == MERGE_CONFIG_EXTENSION_NAME => merge_config_path.unwrap_or_else(|| source.into()),
		Some(source) => source.into(),
		None => ".millenniumrc".into()
	};

	if config_.millennium.bundle.identifier == "com.millennium.dev" {
		error!(
			"You must change the bundle identifier in `{} > millennium > bundle > identifier`. The default value `com.millennium.dev` is not allowed as it must be unique across applications.",
			bundle_identifier_source
		);
		std::process::exit(1);
	}

	if config_
		.millennium
		.bundle
		.identifier
		.chars()
		.any(|ch| !(ch.is_alphanumeric() || ch == '-' || ch == '.'))
	{
		error!(
			"The bundle identifier defined in `{} > millennium > bundle > identifier` is invalid. Check the documentation for more info.",
			bundle_identifier_source
		);
		std::process::exit(1);
	}

	if let Some(before_build) = &config_.build.before_build_command {
		if !before_build.is_empty() {
			info!(action = "Running"; "beforeBuildCommand `{}`", before_build);
			#[cfg(target_os = "windows")]
			let status = Command::new("cmd")
				.arg("/S")
				.arg("/C")
				.arg(before_build)
				.current_dir(app_dir())
				.envs(command_env(options.debug))
				.piped()
				.with_context(|| format!("failed to run `{}` with `cmd /C`", before_build))?;
			#[cfg(not(target_os = "windows"))]
			let status = Command::new("sh")
				.arg("-c")
				.arg(before_build)
				.current_dir(app_dir())
				.envs(command_env(options.debug))
				.piped()
				.with_context(|| format!("failed to run `{}` with `sh -c`", before_build))?;

			if !status.success() {
				bail!("beforeBuildCommand `{}` failed with exit code {}", before_build, status.code().unwrap_or_default());
			}
		}
	}

	if let AppUrl::Url(WindowUrl::App(web_asset_path)) = &config_.build.dist_dir {
		if !web_asset_path.exists() {
			return Err(anyhow::anyhow!(
				"Unable to find your web assets, did you forget to build your web app? Your distDir is set to \"{:?}\".",
				web_asset_path
			));
		}
		if web_asset_path.canonicalize()?.file_name() == Some(std::ffi::OsStr::new("src")) {
			return Err(anyhow::anyhow!(
				"The configured distDir is the `src` folder.
            Please isolate your web assets on a separate folder and update `.millenniumrc > build > distDir`.",
			));
		}

		let mut out_folders = Vec::new();
		for folder in &["node_modules", "src", "target"] {
			if web_asset_path.join(folder).is_dir() {
				out_folders.push(folder.to_string());
			}
		}
		if !out_folders.is_empty() {
			return Err(anyhow::anyhow!(
				"The configured distDir includes the `{:?}` {}. Please isolate your web assets on a separate folder and update `.millenniumrc > build > distDir`.",
				out_folders,
				if out_folders.len() == 1 { "folder" } else { "folders" }
			));
		}
	}

	if options.runner.is_none() {
		options.runner = config_.build.runner.clone();
	}

	if let Some(list) = options.features.as_mut() {
		list.extend(config_.build.features.clone().unwrap_or_default());
	}

	let mut interface = AppInterface::new(config_)?;
	let app_settings = interface.app_settings();
	let interface_options = options.clone().into();

	let bin_path = app_settings.app_binary_path(&interface_options)?;
	let out_dir = bin_path.parent().unwrap();

	interface.build(interface_options)?;

	let app_settings = interface.app_settings();

	if config_.millennium.bundle.active {
		let package_types = if let Some(names) = &options.bundles {
			let mut types = vec![];
			for name in names.iter().flat_map(|n| n.split(',').map(|s| s.to_string()).collect::<Vec<String>>()) {
				if name == "none" {
					break;
				}
				match PackageType::from_short_name(&name) {
					Some(package_type) => {
						types.push(package_type);
					}
					None => {
						return Err(anyhow::anyhow!(format!("Unsupported bundle format: {}", name)));
					}
				}
			}
			Some(types)
		} else {
			let targets = config_.millennium.bundle.targets.to_vec();
			if targets.is_empty() { None } else { Some(targets.into_iter().map(Into::into).collect()) }
		};

		if let Some(types) = &package_types {
			if config_.millennium.updater.active && !types.contains(&PackageType::Updater) {
				warn!("Updater is enabled, but the bundle target list does not contain `updater`; updater artifacts won't be generated.");
			}
		}

		let settings = app_settings
			.get_bundler_settings(&options.into(), config_, out_dir, package_types)
			.with_context(|| "failed to build bundler settings")?;

		// set env vars used by the bundler
		#[cfg(target_os = "linux")]
		{
			use crate::helpers::config::ShellAllowlistOpen;
			if matches!(config_.millennium.allowlist.shell.open, ShellAllowlistOpen::Flag(true) | ShellAllowlistOpen::Validate(_)) {
				std::env::set_var("APPIMAGE_BUNDLE_XDG_OPEN", "1");
			}
			if config_.millennium.system_tray.is_some() {
				if let Ok(tray) = std::env::var("MILLENNIUM_TRAY") {
					std::env::set_var(
						"TRAY_LIBRARY_PATH",
						if tray == "ayatana" {
							format!(
								"{}/libayatana-appindicator3.so.1",
								pkgconfig_utils::get_library_path("ayatana-appindicator3-0.1")
									.expect("failed to get ayatana-appindicator library path using pkg-config.")
							)
						} else {
							format!(
								"{}/libappindicator3.so.1",
								pkgconfig_utils::get_library_path("appindicator3-0.1")
									.expect("failed to get libappindicator-gtk library path using pkg-config.")
							)
						}
					);
				} else {
					std::env::set_var("TRAY_LIBRARY_PATH", pkgconfig_utils::get_appindicator_library_path());
				}
			}

			if config_.millennium.bundle.appimage.bundle_media_framework {
				std::env::set_var("APPIMAGE_BUNDLE_GSTREAMER", "1");
			}
		}

		let bundles = bundle_project(settings).with_context(|| "failed to bundle project")?;

		// If updater is active
		if config_.millennium.updater.active {
			// make sure we have our package builts
			let mut signed_paths = Vec::new();
			for elem in bundles.iter().filter(|bundle| bundle.package_type == PackageType::Updater) {
				// we expect to have only one path in the vec but we iter if we add
				// another type of updater package who require multiple file signature
				for path in elem.bundle_paths.iter() {
					// sign our path from environment variables
					let (signature_path, _signature) = sign_file_from_env_variables(path)?;
					signed_paths.append(&mut vec![signature_path]);
				}
			}

			if !signed_paths.is_empty() {
				print_signed_updater_archive(&signed_paths)?;
			}
		}
	}

	Ok(())
}

fn print_signed_updater_archive(output_paths: &[PathBuf]) -> crate::Result<()> {
	let pluralised = if output_paths.len() == 1 { "updater archive" } else { "updater archives" };
	let msg = format!("{} {} at:", output_paths.len(), pluralised);
	info!("{}", msg);
	for path in output_paths {
		info!("        {}", path.display());
	}
	Ok(())
}

#[cfg(target_os = "linux")]
mod pkgconfig_utils {
	use std::{path::PathBuf, process::Command};

	pub fn get_appindicator_library_path() -> PathBuf {
		match get_library_path("ayatana-appindicator3-0.1") {
			Some(p) => format!("{}/libayatana-appindicator3.so.1", p).into(),
			None => match get_library_path("appindicator3-0.1") {
				Some(p) => format!("{}/libappindicator3.so.1", p).into(),
				None => panic!("Can't detect any appindicator library")
			}
		}
	}

	/// Gets the folder in which a library is located using `pkg-config`.
	pub fn get_library_path(name: &str) -> Option<String> {
		let mut cmd = Command::new("pkg-config");
		cmd.env("PKG_CONFIG_ALLOW_SYSTEM_LIBS", "1");
		cmd.arg("--libs-only-L");
		cmd.arg(name);
		if let Ok(output) = cmd.output() {
			if !output.stdout.is_empty() {
				// output would be "-L/path/to/library\n"
				let word = output.stdout[2..].to_vec();
				return Some(String::from_utf8_lossy(&word).trim().to_string());
			} else {
				None
			}
		} else {
			None
		}
	}
}
