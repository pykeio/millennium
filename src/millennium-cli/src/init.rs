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
	collections::BTreeMap,
	env::current_dir,
	fmt::Display,
	fs::{read_to_string, remove_dir_all},
	path::PathBuf,
	str::FromStr
};

use anyhow::Context;
use clap::Parser;
use dialoguer::Input;
use handlebars::{to_json, Handlebars};
use include_dir::{include_dir, Dir};
use serde::Deserialize;

use crate::Result;
use crate::{
	helpers::{
		framework::{infer_from_package_json as infer_framework, Framework},
		resolve_millennium_path, template, Logger
	},
	VersionMetadata
};

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[derive(Debug, Parser)]
#[clap(about = "Initializes a Millennium project from a pre-made template")]
pub struct Options {
	/// Skip prompting for values
	#[clap(long)]
	ci: bool,
	/// Force init to overwrite the src folder
	#[clap(short, long)]
	force: bool,
	/// Enables logging
	#[clap(short, long)]
	log: bool,
	/// Set target directory for init
	#[clap(short, long)]
	#[clap(default_value_t = current_dir().expect("failed to read cwd").display().to_string())]
	directory: String,
	/// Path of the Millennium project to use (relative to the cwd)
	#[clap(short, long)]
	millennium_path: Option<PathBuf>,
	/// Name of your Millennium application
	#[clap(short = 'A', long)]
	app_name: Option<String>,
	/// Window title of your Millennium application
	#[clap(short = 'W', long)]
	window_title: Option<String>,
	/// Web assets location, relative to <project-dir>
	#[clap(short = 'D', long)]
	dist_dir: Option<String>,
	/// Url of your dev server
	#[clap(short = 'P', long)]
	dev_path: Option<String>
}

#[derive(Deserialize)]
struct PackageJson {
	name: Option<String>,
	product_name: Option<String>
}

#[derive(Default)]
struct InitDefaults {
	app_name: Option<String>,
	framework: Option<Framework>
}

impl Options {
	fn load(mut self) -> Result<Self> {
		self.ci = self.ci || std::env::var("CI").is_ok();
		let package_json_path = PathBuf::from(&self.directory).join("package.json");

		let init_defaults = if package_json_path.exists() {
			let package_json_text = read_to_string(package_json_path)?;
			let package_json: PackageJson = serde_json::from_str(&package_json_text)?;
			let (framework, _) = infer_framework(&package_json_text);
			InitDefaults {
				app_name: package_json.product_name.or(package_json.name),
				framework
			}
		} else {
			Default::default()
		};

		self.app_name = self
			.app_name
			.map(|s| Ok(Some(s)))
			.unwrap_or_else(|| request_input("What is the name of your app?", init_defaults.app_name.clone(), self.ci))?;

		self.window_title = self
			.window_title
			.map(|s| Ok(Some(s)))
			.unwrap_or_else(|| request_input("What should the window title be?", init_defaults.app_name.clone(), self.ci))?;

		self.dist_dir = self.dist_dir.map(|s| Ok(Some(s))).unwrap_or_else(|| {
			request_input(
				r#"Where are your web assets (HTML/CSS/JS) located, relative to <current dir>? (usually dist)"#,
				init_defaults.framework.as_ref().map(|f| f.dist_dir()),
				self.ci
			)
		})?;

		self.dev_path = self.dev_path.map(|s| Ok(Some(s))).unwrap_or_else(|| {
			request_input("What is the URL of your development server? (for hot reloading)", init_defaults.framework.map(|f| f.dev_path()), self.ci)
		})?;

		Ok(self)
	}
}

pub fn command(mut options: Options) -> Result<()> {
	options = options.load()?;
	let logger = Logger::new("millennium:init");

	let template_target_path = PathBuf::from(&options.directory);
	let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../metadata.json"))?;

	if template_target_path.exists() && template_target_path.read_dir().map(|mut i| i.next().is_none()).unwrap_or(false) && !options.force {
		logger.warn(format!("Directory ({:?}) not empty. Run `init --force` to overwrite.", template_target_path));
	} else {
		let (millennium_dep, millennium_build_dep) = if let Some(millennium_path) = options.millennium_path {
			(
				format!(r#"{{  path = {:?}, features = [ "api-all" ] }}"#, resolve_millennium_path(&millennium_path, "src/millennium")),
				format!("{{  path = {:?} }}", resolve_millennium_path(&millennium_path, "src/millennium-build"))
			)
		} else {
			(
				format!(r#"{{ version = "{}", features = [ "api-all" ] }}"#, metadata.millennium),
				format!(r#"{{ version = "{}" }}"#, metadata.millennium_build)
			)
		};

		let _ = remove_dir_all(&template_target_path);
		let handlebars = Handlebars::new();

		let mut data = BTreeMap::new();
		data.insert("millennium_dep", to_json(millennium_dep));
		data.insert("millennium_build_dep", to_json(millennium_build_dep));
		data.insert("dist_dir", to_json(options.dist_dir.unwrap_or_else(|| "../dist".to_string())));
		data.insert("dev_path", to_json(options.dev_path.unwrap_or_else(|| "http://localhost:7216".to_string())));
		data.insert("app_name", to_json(options.app_name.unwrap_or_else(|| "Millennium App".to_string())));
		data.insert("window_title", to_json(options.window_title.unwrap_or_else(|| "Millennium App".to_string())));

		template::render(&handlebars, &data, &TEMPLATE_DIR, &options.directory).with_context(|| "failed to render Millennium template")?;
	}

	Ok(())
}

fn request_input<T>(prompt: &str, default: Option<T>, skip: bool) -> Result<Option<T>>
where
	T: Clone + FromStr + Display + ToString,
	T::Err: Display + std::fmt::Debug
{
	if skip {
		Ok(default)
	} else {
		let theme = dialoguer::theme::ColorfulTheme::default();
		let mut builder = Input::with_theme(&theme);
		builder.with_prompt(prompt);

		if let Some(v) = default {
			builder.default(v.clone());
			builder.with_initial_text(v.to_string());
		}

		builder.interact_text().map(Some).map_err(Into::into)
	}
}
