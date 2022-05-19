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

use std::fs::{create_dir_all, File};
use std::io::Write;
use std::{collections::BTreeMap, env::current_dir, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use handlebars::{to_json, Handlebars};
use include_dir::{include_dir, Dir};
use inquire::{Select, Text};

use crate::helpers::template::Template;
use crate::Result;
use crate::{
	helpers::{resolve_millennium_path, template, Logger},
	VersionMetadata
};

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");
const MILLENNIUMRC_TEMPLATE: &str = include_str!("../templates/.millenniumrc");

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
	/// Name of the template to use
	#[clap(short, long)]
	template: Option<Template>,
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

impl Options {
	fn load(mut self) -> Result<Self> {
		self.ci = self.ci || std::env::var("CI").is_ok();

		self.template = self.template.map(|s| Ok(Some(s))).unwrap_or_else(|| {
			let text = Select::new("What template would you like to use?", Template::VARIANTS.to_vec());
			match text.prompt() {
				Ok(name) => Ok(Some(name)),
				Err(e) => Err(e)
			}
		})?;

		self.app_name = self.app_name.map(|s| Ok(Some(s))).unwrap_or_else(|| {
			let text = Text::new("What is the name of your app?")
				.with_help_message("This is the identifier of your app and should contain only alphanumeric characters, underscores, and dashes.");
			match text.prompt() {
				Ok(name) => Ok(Some(name)),
				Err(e) => Err(e)
			}
		})?;

		self.window_title = self.window_title.map(|s| Ok(Some(s))).unwrap_or_else(|| {
			let text = Text::new("What should the window title be?")
				.with_help_message("This is the human-readable name of your app that will be shown as the window title and can contain any characters.");
			match text.prompt() {
				Ok(name) => Ok(Some(name)),
				Err(e) => Err(e)
			}
		})?;

		self.dist_dir = self.dist_dir.map(|s| Ok(Some(s))).unwrap_or_else(|| {
			let text = Text::new("Where are your web assets (HTML/CSS/JS) located?")
				.with_default("dist")
				.with_help_message(
					"This is the path to your compiled web assets relative to the app root (usually dist), or for static sites, the path to the site contents."
				);
			match text.prompt() {
				Ok(name) => Ok(Some(name)),
				Err(e) => Err(e)
			}
		})?;

		self.dev_path = self.dev_path.map(|s| Ok(Some(s))).unwrap_or_else(|| {
			let text = Text::new("What is the URL of your development server?")
				.with_default("http://localhost:3000")
				.with_help_message("If you're using a build tool with support for hot reloading, this is the URL of your dev server.\nOtherwise, set this to be the same path as your web assets entered previously.");
			match text.prompt() {
				Ok(name) => Ok(Some(name)),
				Err(e) => Err(e)
			}
		})?;

		Ok(self)
	}
}

pub fn command(mut options: Options) -> Result<()> {
	options = options.load()?;
	let logger = Logger::new("millennium:init");

	let template_target_path = PathBuf::from(&options.directory);
	let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../metadata.json"))?;

	if template_target_path.exists() && !template_target_path.read_dir().map(|mut i| i.next().is_none()).unwrap_or(false) && !options.force {
		logger.warn(format!("Target directory ({:?}) is not empty. Run `init --force` to overwrite.", template_target_path));
	} else {
		let (millennium_dep, millennium_build_dep) = if let Some(millennium_path) = options.millennium_path {
			(
				format!(r#"{{ path = {:?}, features = [ "api-all" ] }}"#, resolve_millennium_path(&millennium_path, "src/millennium")),
				format!("{{ path = {:?} }}", resolve_millennium_path(&millennium_path, "src/millennium-build"))
			)
		} else {
			(
				format!(r#"{{ version = "{}", features = [ "api-all" ] }}"#, metadata.millennium),
				format!(r#"{{ version = "{}" }}"#, metadata.millennium_build)
			)
		};

		let handlebars = Handlebars::new();

		let mut data = BTreeMap::new();
		data.insert("millennium_dep", to_json(millennium_dep));
		data.insert("millennium_build_dep", to_json(millennium_build_dep));
		data.insert("dist_dir", to_json(options.dist_dir.unwrap_or_else(|| "../dist".to_string())));
		data.insert("dev_path", to_json(options.dev_path.unwrap_or_else(|| "http://localhost:7216".to_string())));
		data.insert("app_name", to_json(options.app_name.unwrap_or_else(|| "Millennium App".to_string())));
		data.insert("window_title", to_json(options.window_title.unwrap_or_else(|| "Millennium App".to_string())));

		let config = &handlebars
			.render_template(MILLENNIUMRC_TEMPLATE, &data)
			.expect("Failed to render .millenniumrc template");
		data.insert("millennium_config", to_json(config));

		let template_id = options.template.unwrap_or(Template::Basic).id();
		let template_id = template_id.as_str();
		template::render(&handlebars, &data, TEMPLATE_DIR.get_dir(template_id).unwrap(), &options.directory, template_id)
			.with_context(|| "failed to render Millennium template")?;

		create_dir_all(PathBuf::from(&options.directory).join("icons"))?;
		for file in TEMPLATE_DIR.get_dir(".icons").unwrap().files() {
			let mut output_file = File::create(PathBuf::from(&options.directory).join("icons").join(file.path().file_name().unwrap()))?;
			output_file.write_all(file.contents())?;
		}
	}

	println!("{}", "Your app is ready! Happy coding! 🎉".bold().blue());
	Ok(())
}
