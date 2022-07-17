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

use std::{
	collections::HashMap,
	env::set_var,
	process::exit,
	sync::{Arc, Mutex}
};

use anyhow::Context;
use json_patch::merge;
pub use millennium_utils::config::*;
use once_cell::sync::Lazy;
use serde_json::Value as JsonValue;

pub const MERGE_CONFIG_EXTENSION_NAME: &str = "--config";

pub struct ConfigMetadata {
	/// The actual configuration, merged with any extension.
	inner: Config,
	/// A list of config extensions (i.e. platform-specific config files or an extra config file provided by the
	/// `--config` CLI argument), mapped by extension name to its value.
	extensions: HashMap<&'static str, JsonValue>
}

impl std::ops::Deref for ConfigMetadata {
	type Target = Config;

	#[inline(always)]
	fn deref(&self) -> &Config {
		&self.inner
	}
}

impl ConfigMetadata {
	/// Checks which config is overwriting the bundle identifier.
	pub fn find_bundle_identifier_override(&self) -> Option<&'static str> {
		for (ext, config) in &self.extensions {
			if let Some(identifier) = config
				.as_object()
				.and_then(|config| config.get("millennium"))
				.and_then(|millennium_config| millennium_config.as_object())
				.and_then(|millennium_config| millennium_config.get("bundle"))
				.and_then(|bundle_config| bundle_config.as_object())
				.and_then(|bundle_config| bundle_config.get("identifier"))
				.and_then(|id| id.as_str())
			{
				if identifier == self.inner.millennium.bundle.identifier {
					return Some(ext);
				}
			}
		}
		None
	}
}

pub type ConfigHandle = Arc<Mutex<Option<ConfigMetadata>>>;

pub fn wix_settings(config: WixConfig) -> millennium_bundler::WixSettings {
	millennium_bundler::WixSettings {
		language: millennium_bundler::WixLanguage(match config.language {
			WixLanguage::One(lang) => vec![(lang, Default::default())],
			WixLanguage::List(languages) => languages.into_iter().map(|lang| (lang, Default::default())).collect(),
			WixLanguage::Localized(languages) => languages
				.into_iter()
				.map(|(lang, config)| {
					(
						lang,
						millennium_bundler::WixLanguageConfig {
							locale_path: config.locale_path.map(Into::into)
						}
					)
				})
				.collect()
		}),
		template: config.template,
		fragment_paths: config.fragment_paths,
		component_group_refs: config.component_group_refs,
		component_refs: config.component_refs,
		feature_group_refs: config.feature_group_refs,
		feature_refs: config.feature_refs,
		merge_refs: config.merge_refs,
		skip_webview_install: config.skip_webview_install,
		license: config.license,
		enable_elevated_update_task: config.enable_elevated_update_task,
		banner_path: config.banner_path,
		dialog_image_path: config.dialog_image_path
	}
}

fn config_handle() -> &'static ConfigHandle {
	static CONFING_HANDLE: Lazy<ConfigHandle> = Lazy::new(Default::default);
	&CONFING_HANDLE
}

/// Gets the static parsed config from `.millenniumrc`.
fn get_internal(merge_config: Option<&str>, reload: bool) -> crate::Result<ConfigHandle> {
	if !reload && config_handle().lock().unwrap().is_some() {
		return Ok(config_handle().clone());
	}

	let millennium_dir = super::app_paths::millennium_dir();
	let mut config = millennium_utils::config::parse::parse_value(millennium_dir.join(".millenniumrc"))?;
	let mut extensions = HashMap::new();

	if let Some(platform_config) = millennium_utils::config::parse::read_platform(millennium_dir)? {
		merge(&mut config, &platform_config);
		extensions.insert(millennium_utils::config::parse::get_platform_config_filename(), platform_config);
	}

	if let Some(merge_config) = merge_config {
		set_var("MILLENNIUM_CONFIG", serde_json::to_string(&config)?);

		let merge_config: JsonValue = serde_json::from_str(merge_config).with_context(|| "failed to parse config to merge")?;
		merge(&mut config, &merge_config);
		extensions.insert(MERGE_CONFIG_EXTENSION_NAME, merge_config);
	}

	let schema: JsonValue = serde_json::from_str(include_str!("../../schema.json"))?;
	let mut scope = valico::json_schema::Scope::new();
	let schema = scope.compile_and_return(schema, false).unwrap();
	let state = schema.validate(&config);
	if !state.errors.is_empty() {
		for error in state.errors {
			let path = error.get_path().chars().skip(1).collect::<String>().replace('/', " > ");
			if path.is_empty() {
				eprintln!("`.millenniumrc` error: {}", error.get_detail().unwrap_or_else(|| error.get_title()));
			} else {
				eprintln!("`.millenniumrc` error in `{}`: {}", path, error.get_detail().unwrap_or_else(|| error.get_title()));
			}
		}
		exit(1);
	}

	let config: Config = serde_json::from_value(config)?;

	*config_handle().lock().unwrap() = Some(ConfigMetadata { inner: config, extensions });
	Ok(config_handle().clone())
}

pub fn get(merge_config: Option<&str>) -> crate::Result<ConfigHandle> {
	get_internal(merge_config, false)
}

pub fn reload(merge_config: Option<&str>) -> crate::Result<ConfigHandle> {
	get_internal(merge_config, true)
}
