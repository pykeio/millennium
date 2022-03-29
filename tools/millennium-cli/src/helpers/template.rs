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
	fmt::{Display, Formatter},
	fs::{create_dir_all, File},
	io::Write,
	path::Path,
	str::FromStr
};

use handlebars::Handlebars;
use include_dir::Dir;

#[derive(Debug, Copy, Clone)]
pub enum Template {
	Basic,
	Intermediate,
	Advanced,
	BasicCxx,
	React,
	Preact,
	PreactWmr,
	Svelte,
	Vue
}

impl Template {
	pub const VARIANTS: &'static [Template] = &[
		Template::Basic,
		Template::Intermediate,
		Template::Advanced,
		Template::BasicCxx,
		Template::React,
		Template::Preact,
		Template::PreactWmr,
		Template::Svelte,
		Template::Vue
	];

	pub fn id(self) -> String {
		match self {
			Template::Basic => "basic",
			Template::Intermediate => "intermediate",
			Template::Advanced => "advanced",
			Template::BasicCxx => "basic-cxx",
			Template::React => "react",
			Template::Preact => "preact",
			Template::PreactWmr => "preact-wmr",
			Template::Svelte => "svelte",
			Template::Vue => "vue"
		}
		.to_string()
	}
}

impl FromStr for Template {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"basic" => Ok(Template::Basic),
			"intermediate" => Ok(Template::Intermediate),
			"advanced" => Ok(Template::Advanced),
			"basic-cxx" => Ok(Template::BasicCxx),
			"react" => Ok(Template::React),
			"preact" => Ok(Template::Preact),
			"preact-wmr" => Ok(Template::PreactWmr),
			"svelte" => Ok(Template::Svelte),
			"vue" => Ok(Template::Vue),
			_ => Err(format!("Unknown template: {}", s))
		}
	}
}

impl Display for Template {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Template::Basic => "Basic",
				Template::Intermediate => "Intermediate",
				Template::Advanced => "Advanced",
				Template::BasicCxx => "Basic (via C++ bindings)",
				Template::React => "React (w/ react-scripts)",
				Template::Preact => "Preact (w/ esbuild)",
				Template::PreactWmr => "Preact (w/ WMR)",
				Template::Svelte => "Svelte",
				Template::Vue => "Vue"
			}
		)
	}
}

pub fn render<P: AsRef<Path>>(
	handlebars: &Handlebars<'_>,
	data: &BTreeMap<&str, serde_json::Value>,
	dir: &Dir<'_>,
	out_dir: P,
	base: &str
) -> crate::Result<()> {
	let dir_path = dir.path().strip_prefix(base).unwrap();
	create_dir_all(out_dir.as_ref().join(dir_path))?;
	for file in dir.files() {
		let mut file_path = file.path().strip_prefix(base).unwrap().to_path_buf();
		// cargo for some reason ignores the /templates folder packaging when it has a Cargo.toml file inside
		// so we rename the extension to `.crate-manifest`
		if let Some(extension) = file_path.extension() {
			if extension == "crate-manifest" {
				file_path.set_extension("toml");
			}
		}
		let mut output_file = File::create(out_dir.as_ref().join(file_path))?;
		if let Some(utf8) = file.contents_utf8() {
			handlebars
				.render_template_to_write(utf8, &data, &mut output_file)
				.expect("Failed to render template");
		} else {
			output_file.write_all(file.contents())?;
		}
	}
	for dir in dir.dirs() {
		render(handlebars, data, dir, out_dir.as_ref(), base)?;
	}
	Ok(())
}
