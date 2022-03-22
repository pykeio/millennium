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

use std::path::PathBuf;

use clap::Parser;

use crate::{
	helpers::updater_signature::{generate_key, save_keypair},
	Result
};

#[derive(Debug, Parser)]
#[clap(about = "Generate keypair to sign files")]
pub struct Options {
	/// Set private key password when signing
	#[clap(short, long)]
	password: Option<String>,
	/// Write private key to a file
	#[clap(short, long)]
	write_keys: Option<PathBuf>,
	/// Overwrite private key even if it exists on the specified path
	#[clap(short, long)]
	force: bool
}

pub fn command(options: Options) -> Result<()> {
	if options.password.is_none() {
		println!("Generating new private key without password.")
	}
	let keypair = generate_key(options.password).expect("Failed to generate key");

	if let Some(output_path) = options.write_keys {
		let (secret_path, public_path) = save_keypair(options.force, output_path, &keypair.sk, &keypair.pk).expect("Unable to write keypair");

		println!(
			"\nYour keypair was generated successfully\nPrivate: {} (Keep it secret!)\nPublic: {}\n---------------------------",
			secret_path.display(),
			public_path.display()
		)
	} else {
		println!("\nYour secret key was generated successfully - Keep it secret!\n{}\n\n", keypair.sk);
		println!("Your public key was generated successfully:\n{}\n\nAdd the public key in your .millenniumrc:\n---------------------------\n", keypair.pk);
	}

	println!(
		"\nEnvironment variabled used to sign:\n`MILLENNIUM_PRIVATE_KEY`  Path or String of your private key\n`MILLENNIUM_KEY_PASSWORD`  Your private key password (optional)\n\nATTENTION: If you lose your private key OR password, you'll not be able to sign your update package and updates will not works.\n---------------------------\n"
	);

	Ok(())
}
