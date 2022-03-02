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

//! The Millennium API interface.

#[cfg(feature = "dialog")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "dialog")))]
pub mod dialog;
pub mod dir;
pub mod file;
#[cfg(feature = "http-api")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "http-api")))]
pub mod http;
pub mod ipc;
pub mod path;
pub mod process;
#[cfg(feature = "shell-open-api")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "shell-open-api")))]
pub mod shell;
pub mod version;

#[cfg(feature = "cli")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "cli")))]
pub mod cli;

#[cfg(feature = "cli")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "cli")))]
pub use clap;

#[cfg(feature = "notification")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "notification")))]
pub mod notification;

mod error;

/// The error type of the Millennium API module.
pub use error::Error;
/// The result type of the Millennium API module.
pub type Result<T> = std::result::Result<T, Error>;

// Not public API
#[doc(hidden)]
pub mod private {
	pub use once_cell::sync::OnceCell;

	pub trait AsMillenniumContext {
		fn config() -> &'static crate::Config;
		fn assets() -> &'static crate::utils::assets::EmbeddedAssets;
		fn default_window_icon() -> Option<&'static [u8]>;
		fn package_info() -> crate::PackageInfo;
	}
}
