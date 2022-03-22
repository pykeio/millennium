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

pub use anyhow::Result;

mod build;
mod dev;
mod helpers;
mod info;
mod init;
mod interface;
mod plugin;
mod signer;

use std::ffi::OsString;

use clap::{FromArgMatches, IntoApp, Parser, Subcommand};

pub(crate) trait CommandExt {
	fn pipe(&mut self) -> Result<&mut Self>;
}

impl CommandExt for std::process::Command {
	fn pipe(&mut self) -> Result<&mut Self> {
		self.stdout(os_pipe::dup_stdout()?);
		self.stderr(os_pipe::dup_stderr()?);
		Ok(self)
	}
}

#[derive(serde::Deserialize)]
pub struct VersionMetadata {
	millennium: String,
	#[serde(rename = "millennium-build")]
	millennium_build: String
}

#[derive(Parser)]
#[clap(
	author,
	version,
	about,
	bin_name("cargo-millennium"),
	subcommand_required(true),
	arg_required_else_help(true),
	propagate_version(true),
	no_binary_name(true)
)]
struct Cli {
	#[clap(subcommand)]
	command: Commands
}

#[derive(Subcommand)]
enum Commands {
	Build(build::Options),
	Dev(dev::Options),
	Info(info::Options),
	Init(init::Options),
	Plugin(plugin::Cli),
	Signer(signer::Cli)
}

fn format_error<I: IntoApp>(err: clap::Error) -> clap::Error {
	let mut app = I::command();
	err.format(&mut app)
}

/// Run the Millennium CLI with the passed arguments.
///
/// The passed arguments should have the binary argument(s) stripped out before being passed.
///
/// e.g.
/// 1. `millennium-cli 1 2 3` -> `1 2 3`
/// 2. `cargo millennium 1 2 3` -> `1 2 3`
/// 3. `node millennium.js 1 2 3` -> `1 2 3`
///
/// The passed `bin_name` parameter should be how you want the help messages to display the command.
/// This defaults to `cargo-millennium`, but should be set to how the program was called, such as
/// `cargo millennium`.
pub fn run<I, A>(args: I, bin_name: Option<String>) -> Result<()>
where
	I: IntoIterator<Item = A>,
	A: Into<OsString> + Clone
{
	let matches = match bin_name {
		Some(bin_name) => Cli::command().bin_name(bin_name),
		None => Cli::command()
	}
	.get_matches_from(args);

	let res = Cli::from_arg_matches(&matches).map_err(format_error::<Cli>);
	let cli = match res {
		Ok(s) => s,
		Err(e) => e.exit()
	};

	match cli.command {
		Commands::Build(options) => build::command(options)?,
		Commands::Dev(options) => dev::command(options)?,
		Commands::Info(options) => info::command(options)?,
		Commands::Init(options) => init::command(options)?,
		Commands::Plugin(cli) => plugin::command(cli)?,
		Commands::Signer(cli) => signer::command(cli)?
	}

	Ok(())
}
