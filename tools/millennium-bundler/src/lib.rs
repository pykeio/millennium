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

#![warn(missing_docs, rust_2018_idioms)]

//! Millennium Bundler is a tool that generates installers or app bundles for executables.
//!
//! # Platform support
//! - macOS
//!   - DMG and App bundles
//! - Linux
//!   - Appimage and Debian packages
//! - Windows
//!   - MSI using WiX

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

/// The bundle API.
pub mod bundle;
mod error;
pub use bundle::*;
pub use error::{Error, Result};
