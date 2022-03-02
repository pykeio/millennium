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

mod fs;
mod http;
#[cfg(shell_scope)]
mod shell;

pub use fs::Scope as FsScope;
#[cfg(shell_scope)]
pub use shell::{
	ExecuteArgs, Scope as ShellScope, ScopeAllowedArg as ShellScopeAllowedArg, ScopeAllowedCommand as ShellScopeAllowedCommand,
	ScopeConfig as ShellScopeConfig, ScopeError as ShellScopeError
};

pub use self::http::Scope as HttpScope;

pub(crate) struct Scopes {
	pub fs: FsScope,
	#[cfg(protocol_asset)]
	pub asset_protocol: FsScope,
	#[cfg(http_request)]
	pub http: HttpScope,
	#[cfg(shell_scope)]
	pub shell: ShellScope
}
