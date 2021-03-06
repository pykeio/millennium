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

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use json_patch::merge;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

use crate::config::Config;

/// All file extensions that are supported.
pub const EXTENSIONS_SUPPORTED: &[&str] = &["json", "json5", "jsonc"];

/// Represents all the errors that can happen while reading the config.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
	/// Failed to parse the config file in JSON5 format.
	#[error("unable to parse Millennium config file at {path} because {error}")]
	FormatJson {
		/// The path that failed to parse into JSON5.
		path: PathBuf,

		/// The parsing [`json5::Error`].
		error: ::json5::Error
	},

	/// Unknown file extension encountered.
	#[error("unsupported format encountered {0}")]
	UnsupportedFormat(String),

	/// A generic IO error with context of what caused it.
	#[error("unable to read Millennium config file at {path} because {error}")]
	Io {
		/// The path the IO error occured on.
		path: PathBuf,

		/// The [`std::io::Error`].
		error: std::io::Error
	}
}

/// Reads the configuration from the given root directory.
///
/// It first looks for a `.millenniumrc` file on the given directory. The file
/// must exist. Then it looks for a platform-specific configuration file:
/// - `.millenniumrc.macos` on macOS
/// - `.millenniumrc.linux` on Linux
/// - `.millenniumrc.windows` on Windows
/// Merging the configurations using [JSON Merge Patch (RFC 7396)].
///
/// [JSON Merge Patch (RFC 7396)]: https://datatracker.ietf.org/doc/html/rfc7396.
pub fn read_from(root_dir: PathBuf) -> Result<Value, ConfigError> {
	let mut config: Value = parse_value(root_dir.join(".millenniumrc"))?;
	if let Some(platform_config) = read_platform(root_dir)? {
		merge(&mut config, &platform_config);
	}
	Ok(config)
}

/// Gets the platform configuration file name.
pub fn get_platform_config_filename() -> &'static str {
	if cfg!(target_os = "macos") {
		".millenniumrc.macos"
	} else if cfg!(windows) {
		".millenniumrc.windows"
	} else {
		".millenniumrc.linux"
	}
}

/// Reads the platform-specific configuration file in the given directory.
pub fn read_platform(root_dir: PathBuf) -> Result<Option<Value>, ConfigError> {
	let platform_config_path = root_dir.join(get_platform_config_filename());
	if does_supported_extension_exist(&platform_config_path) {
		let platform_config: Value = parse_value(platform_config_path)?;
		Ok(Some(platform_config))
	} else {
		Ok(None)
	}
}

/// Check if a supported config file exists at path.
///
/// The passed path is expected to be the path to the "default" configuration
/// format, in this case JSON with `.json`.
pub fn does_supported_extension_exist(path: impl Into<PathBuf>) -> bool {
	let path = path.into();
	path.exists() || EXTENSIONS_SUPPORTED.iter().any(|ext| path.with_extension(ext).exists())
}

/// Parse the config from path, including alternative formats.
///
/// Hierarchy:
/// 1. Check if `.millenniumrc` or `.millenniumrc.json` exists
///   a. Parse it with `serde_json`
///   b. Parse it with `json5` if `serde_json` fails
///   c. Return original `serde_json` error if all above steps failed
/// 2. Check if `.millenniumrc.json5` exists
///   a. Parse it with `json5`
///   b. Return error if all above steps failed
/// 3. Return error if all above steps failed
pub fn parse(path: impl Into<PathBuf>) -> Result<Config, ConfigError> {
	do_parse(path.into())
}

/// See [`parse`] for specifics, returns a JSON [`Value`] instead of [`Config`].
pub fn parse_value(path: impl Into<PathBuf>) -> Result<Value, ConfigError> {
	do_parse(path.into())
}

fn do_parse<D: DeserializeOwned>(path: PathBuf) -> Result<D, ConfigError> {
	let json5 = path.with_extension("json5");
	let path_ext = path.extension().map(OsStr::to_string_lossy).unwrap_or_default();

	if path.exists() {
		let raw = read_to_string(&path)?;
		do_parse_json(&raw, &path)
	} else if json5.exists() {
		let raw = read_to_string(&json5)?;
		do_parse_json(&raw, &path)
	} else if !EXTENSIONS_SUPPORTED.contains(&path_ext.as_ref()) {
		Err(ConfigError::UnsupportedFormat(path_ext.to_string()))
	} else {
		Err(ConfigError::Io {
			path,
			error: std::io::ErrorKind::NotFound.into()
		})
	}
}

/// "Low-level" helper to parse JSON5 into a [`Config`].
///
/// `raw` should be the contents of the file that is represented by `path`.
pub fn parse_json(raw: &str, path: &Path) -> Result<Config, ConfigError> {
	do_parse_json(raw, path)
}

fn do_parse_json<D: DeserializeOwned>(raw: &str, path: &Path) -> Result<D, ConfigError> {
	::json5::from_str(raw).map_err(|error| ConfigError::FormatJson { path: path.into(), error })
}

/// Helper function to wrap IO errors from [`std::fs::read_to_string`] into a
/// [`ConfigError`].
fn read_to_string(path: &Path) -> Result<String, ConfigError> {
	std::fs::read_to_string(path).map_err(|error| ConfigError::Io { path: path.into(), error })
}
