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

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;

use crate::{
	helpers::updater_signature::{read_key_from_file, sign_file},
	Result
};

#[derive(Debug, Parser)]
#[clap(about = "Sign a file")]
pub struct Options {
	/// Load the private key from a file
	#[clap(short = 'k', long, conflicts_with("private-key-path"))]
	private_key: Option<String>,
	/// Load the private key from a string
	#[clap(short = 'f', long, conflicts_with("private-key"))]
	private_key_path: Option<PathBuf>,
	/// Set private key password when signing
	#[clap(short, long)]
	password: Option<String>,
	/// Sign the specified file
	file: PathBuf
}

pub fn command(mut options: Options) -> Result<()> {
	options.private_key = if let Some(private_key) = options.private_key_path {
		Some(read_key_from_file(Path::new(&private_key)).expect("Unable to extract private key"))
	} else {
		options.private_key
	};
	let private_key = if let Some(pk) = options.private_key {
		pk
	} else {
		return Err(anyhow::anyhow!("Key generation aborted: Unable to find the private key".to_string(),));
	};

	if options.password.is_none() {
		println!("Signing without password.");
	}

	let (manifest_dir, signature) = sign_file(private_key, options.password, options.file).with_context(|| "failed to sign file")?;

	println!(
		"\nYour file was signed successfully, You can find the signature here:\n{}\n\nPublic signature:\n{}\n\nMake sure to include this into the signature field of your update server.",
		manifest_dir.display(),
		signature
	);

	Ok(())
}
