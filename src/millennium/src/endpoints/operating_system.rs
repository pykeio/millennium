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

use millennium_macros::{module_command_handler, CommandModule};
use serde::Deserialize;

use super::InvokeContext;
use crate::Runtime;

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
	Platform,
	Version,
	OsType,
	Arch,
	Tempdir
}

impl Cmd {
	#[module_command_handler(os_all, "os > all")]
	fn platform<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
		Ok(os_platform())
	}

	#[module_command_handler(os_all, "os > all")]
	fn version<R: Runtime>(_context: InvokeContext<R>) -> super::Result<String> {
		Ok(os_info::get().version().to_string())
	}

	#[module_command_handler(os_all, "os > all")]
	fn os_type<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
		Ok(os_type())
	}

	#[module_command_handler(os_all, "os > all")]
	fn arch<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
		Ok(std::env::consts::ARCH)
	}

	#[module_command_handler(os_all, "os > all")]
	fn tempdir<R: Runtime>(_context: InvokeContext<R>) -> super::Result<PathBuf> {
		Ok(std::env::temp_dir())
	}
}

#[cfg(os_all)]
fn os_type() -> &'static str {
	#[cfg(target_os = "linux")]
	return "Linux";
	#[cfg(target_os = "windows")]
	return "Windows_NT";
	#[cfg(target_os = "macos")]
	return "Darwin";
}
#[cfg(os_all)]
fn os_platform() -> &'static str {
	match std::env::consts::OS {
		"windows" => "win32",
		"macos" => "darwin",
		_ => std::env::consts::OS
	}
}

#[cfg(test)]
mod tests {
	#[millennium_macros::module_command_test(os_all, "os > all")]
	#[quickcheck_macros::quickcheck]
	fn platform() {}

	#[millennium_macros::module_command_test(os_all, "os > all")]
	#[quickcheck_macros::quickcheck]
	fn version() {}

	#[millennium_macros::module_command_test(os_all, "os > all")]
	#[quickcheck_macros::quickcheck]
	fn os_type() {}

	#[millennium_macros::module_command_test(os_all, "os > all")]
	#[quickcheck_macros::quickcheck]
	fn arch() {}

	#[millennium_macros::module_command_test(os_all, "os > all")]
	#[quickcheck_macros::quickcheck]
	fn tempdir() {}
}
