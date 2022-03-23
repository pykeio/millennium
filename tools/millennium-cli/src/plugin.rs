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

use clap::{Parser, Subcommand};

use crate::Result;

mod init;

#[derive(Parser)]
#[clap(author, version, about = "Manage Millennium plugins", subcommand_required(true), arg_required_else_help(true))]
pub struct Cli {
	#[clap(subcommand)]
	command: Commands
}

#[derive(Subcommand)]
enum Commands {
	Init(init::Options)
}

pub fn command(cli: Cli) -> Result<()> {
	match cli.command {
		Commands::Init(options) => init::command(options)?
	}

	Ok(())
}
