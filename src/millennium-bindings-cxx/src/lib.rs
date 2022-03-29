// Copyright 2022 pyke.io
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

use std::{error::Error, fs, io::Write, path::Path, str};

use regex::Regex;

const RUST_BINDINGS_TEMPLATE: &[u8] = include_bytes!("./template.rs");
const CXX_BINDINGS_TEMPLATE: &[u8] = include_bytes!("./millennium.h");

pub fn build(out_path: &Path, rc_path: &str) -> Result<(), Box<dyn Error>> {
	let mut rust_file = fs::File::create(out_path.join("bindings.rs"))?;
	let mut cxx_file = fs::File::create(out_path.join("millennium.h"))?;

	let rust_code = str::from_utf8(RUST_BINDINGS_TEMPLATE)?;
	rust_file.write_all(Regex::new(r"\$rc_path").unwrap().replace_all(rust_code, rc_path).as_bytes())?;

	let cxx_code = str::from_utf8(CXX_BINDINGS_TEMPLATE)?;
	cxx_file.write_all(Regex::new(r"\$rc_path").unwrap().replace_all(cxx_code, rc_path).as_bytes())?;

	Ok(())
}
