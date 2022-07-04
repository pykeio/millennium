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

use std::env::args_os;
use std::ffi::OsStr;
use std::path::Path;
use std::process::exit;

fn main() {
	let mut args = args_os().peekable();
	let bin_name = match args.next().as_deref().map(Path::new).and_then(Path::file_stem).and_then(OsStr::to_str) {
		Some("cargo-millennium") => {
			if args.peek().and_then(|s| s.to_str()) == Some("millennium") {
				// remove the extra cargo subcommand
				args.next();
				Some("cargo millennium".into())
			} else {
				Some("cargo-millennium".into())
			}
		}
		Some(stem) => Some(stem.to_string()),
		None => {
			eprintln!("cargo-millennium wrapper unable to read first argument");
			exit(1);
		}
	};

	millennium_cli::run(args, bin_name)
}
