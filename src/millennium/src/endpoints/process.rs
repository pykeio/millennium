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

#![allow(unused_imports)]

use millennium_macros::{command_enum, module_command_handler, CommandModule};
use serde::Deserialize;

use super::InvokeContext;
#[cfg(process_relaunch)]
use crate::Manager;
use crate::Runtime;

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
	/// Relaunch application
	Relaunch,
	/// Close application with provided exit_code
	#[cmd(process_exit, "process > exit")]
	#[serde(rename_all = "camelCase")]
	Exit { exit_code: i32 }
}

impl Cmd {
	#[module_command_handler(process_relaunch)]
	fn relaunch<R: Runtime>(context: InvokeContext<R>) -> super::Result<()> {
		context.window.app_handle().restart();
		Ok(())
	}

	#[cfg(not(process_relaunch))]
	fn relaunch<R: Runtime>(_: InvokeContext<R>) -> super::Result<()> {
		Err(crate::Error::ApiNotAllowlisted("process > relaunch".into()).into_anyhow())
	}

	#[module_command_handler(process_exit)]
	fn exit<R: Runtime>(_context: InvokeContext<R>, exit_code: i32) -> super::Result<()> {
		// would be great if we can have a handler inside Millennium
		// who close all window and emit an event that user can catch
		// if they want to process something before closing the app
		std::process::exit(exit_code);
	}
}

#[cfg(test)]
mod tests {
	#[millennium_macros::module_command_test(process_relaunch, "process > relaunch", runtime)]
	#[quickcheck_macros::quickcheck]
	fn relaunch() {}

	#[millennium_macros::module_command_test(process_exit, "process > exit")]
	#[quickcheck_macros::quickcheck]
	fn exit(_exit_code: i32) {}
}
