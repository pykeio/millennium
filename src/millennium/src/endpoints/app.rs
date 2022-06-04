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

use millennium_macros::{command_enum, CommandModule};
use serde::Deserialize;

use super::InvokeContext;
use crate::Runtime;

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
#[allow(clippy::enum_variant_names)]
pub enum Cmd {
	/// Get Application Version
	GetAppVersion,
	/// Get Application Name
	GetAppName,
	/// Get Millennium Version
	GetMillenniumVersion
}

impl Cmd {
	fn get_app_version<R: Runtime>(context: InvokeContext<R>) -> super::Result<String> {
		Ok(context.package_info.version.to_string())
	}

	fn get_app_name<R: Runtime>(context: InvokeContext<R>) -> super::Result<String> {
		Ok(context.package_info.name)
	}

	fn get_millennium_version<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
		Ok(env!("CARGO_PKG_VERSION"))
	}
}
