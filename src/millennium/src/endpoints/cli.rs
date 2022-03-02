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

use millennium_macros::{module_command_handler, CommandModule};
use serde::Deserialize;

use super::{InvokeContext, InvokeResponse};
use crate::Runtime;

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
	/// The get CLI matches API.
	CliMatches
}

impl Cmd {
	#[module_command_handler(cli, "CLI definition not set under .millenniumrc > millennium > cli")]
	fn cli_matches<R: Runtime>(context: InvokeContext<R>) -> super::Result<InvokeResponse> {
		if let Some(cli) = &context.config.millennium.cli {
			crate::api::cli::get_matches(cli, &context.package_info).map(Into::into).map_err(Into::into)
		} else {
			Err(crate::Error::ApiNotAllowlisted("CLI definition not set under .millenniumrc > millennium > cli".into()).into_anyhow())
		}
	}
}

#[cfg(test)]
mod tests {
	#[millennium_macros::module_command_test(cli, "CLI definition not set under .millenniumrc > millennium > cli")]
	#[quickcheck_macros::quickcheck]
	fn cli_matches() {
		let res = super::Cmd::cli_matches(crate::test::mock_invoke_context());
		crate::test_utils::assert_not_allowlist_error(res);
	}
}
