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

use std::{collections::BTreeMap, env::current_dir, fs::remove_dir_all, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use handlebars::{to_json, Handlebars};
use heck::{ToKebabCase, ToSnakeCase};
use include_dir::{include_dir, Dir};
use log::warn;

use crate::Result;
use crate::{
	helpers::{resolve_millennium_path, template},
	VersionMetadata
};

// TODO
const BACKEND_PLUGIN_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/basic");
const API_PLUGIN_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/basic");

#[derive(Debug, Parser)]
#[clap(about = "Initializes a Millennium plugin project")]
pub struct Options {
	/// Name of your Millennium plugin
	#[clap(short = 'n', long = "name")]
	plugin_name: String,
	/// Initializes a Millennium plugin with TypeScript API
	#[clap(long)]
	api: bool,
	/// Initializes a Millennium core plugin (internal usage)
	#[clap(long, hide(true))]
	pyke: bool,
	/// Set target directory for init
	#[clap(short, long)]
	#[clap(default_value_t = current_dir().expect("failed to read cwd").display().to_string())]
	directory: String,
	/// Path of the Millennium project to use (relative to the cwd)
	#[clap(short, long)]
	millennium_path: Option<PathBuf>,
	/// Author name
	#[clap(short, long)]
	author: Option<String>
}

impl Options {
	fn load(&mut self) {
		if self.author.is_none() {
			self.author.replace(if self.pyke { "pykeio".into() } else { "You".into() });
		}
	}
}

pub fn command(mut options: Options) -> Result<()> {
	options.load();
	let template_target_path = PathBuf::from(options.directory).join(&format!("millennium-plugin-{}", options.plugin_name.to_kebab_case()));
	let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../../metadata.json"))?;
	if template_target_path.exists() {
		warn!("Plugin dir ({:?}) not empty.", template_target_path);
	} else {
		let (millennium_dep, millennium_example_dep, millennium_build_dep) = if let Some(millennium_path) = options.millennium_path {
			(
				format!(r#"{{ path = {:?} }}"#, resolve_millennium_path(&millennium_path, "src/millennium")),
				format!(r#"{{ path = {:?}, features = [ "api-all" ] }}"#, resolve_millennium_path(&millennium_path, "src/millennium")),
				format!(r#"{{ path = {:?} }}"#, resolve_millennium_path(&millennium_path, "src/millennium-build"))
			)
		} else {
			(
				format!(r#"{{ version = "{}" }}"#, metadata.millennium),
				format!(r#"{{ version = "{}", features = [ "api-all" ] }}"#, metadata.millennium),
				format!(r#"{{ version = "{}" }}"#, metadata.millennium_build)
			)
		};

		let _ = remove_dir_all(&template_target_path);
		let handlebars = Handlebars::new();

		let mut data = BTreeMap::new();
		data.insert("plugin_name_original", to_json(&options.plugin_name));
		data.insert("plugin_name", to_json(options.plugin_name.to_kebab_case()));
		data.insert("plugin_name_snake_case", to_json(options.plugin_name.to_snake_case()));
		data.insert("millennium_dep", to_json(millennium_dep));
		data.insert("millennium_example_dep", to_json(millennium_example_dep));
		data.insert("millennium_build_dep", to_json(millennium_build_dep));
		data.insert("author", to_json(options.author));

		if options.pyke {
			data.insert(
				"license_template",
				to_json(
					"// Copyright {20\\d{2}(-20\\d{2})?} pyke.io
					//
					// Licensed under the Apache License, Version 2.0 (the \"License\");
					// you may not use this file except in compliance with the License.
					// You may obtain a copy of the License at
					//
					//     http://www.apache.org/licenses/LICENSE-2.0
					//
					// Unless required by applicable law or agreed to in writing, software
					// distributed under the License is distributed on an \"AS IS\" BASIS,
					// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
					// See the License for the specific language governing permissions and
					// limitations under the License.\n\n"
						.replace('\t', "")
						.replace(" //", "//")
				)
			);
			data.insert(
				"license_header",
				to_json(
					"// Copyright {20\\d{2}(-20\\d{2})?} pyke.io
					//
					// Licensed under the Apache License, Version 2.0 (the \"License\");
					// you may not use this file except in compliance with the License.
					// You may obtain a copy of the License at
					//
					//     http://www.apache.org/licenses/LICENSE-2.0
					//
					// Unless required by applicable law or agreed to in writing, software
					// distributed under the License is distributed on an \"AS IS\" BASIS,
					// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
					// See the License for the specific language governing permissions and
					// limitations under the License.\n\n"
						.replace('\t', "")
						.replace(" //", "//")
				)
			);
		}

		template::render(&handlebars, &data, if options.api { &API_PLUGIN_DIR } else { &BACKEND_PLUGIN_DIR }, &template_target_path, "")
			.with_context(|| "failed to render Millennium> template")?;
	}
	Ok(())
}
