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

//! Types and functions related to child processes management.

use std::path::PathBuf;

use crate::Env;

#[cfg(feature = "command")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "command")))]
mod command;
#[cfg(feature = "command")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "command")))]
pub use command::*;

/// Finds the current running binary's path.
///
/// With exception to any following platform-specific behavior, the path is
/// cached as soon as possible, and then used repeatedly instead of querying for
/// a new path every time this function is called.
///
/// # Platform-specific behavior
///
/// ## Linux
///
/// On Linux, this function will **attempt** to detect if it's currently running
/// from a valid [AppImage] and use that path instead.
///
/// ## macOS
///
/// On `macOS`, this function will return an error if the original path
/// contained any symlinks due to less protection on macOS regarding symlinks.
/// This behavior can be disabled by setting the
/// `process-relaunch-dangerous-allow-symlink-macos` feature, although it is
/// *highly discouraged*.
///
/// # Security
///
/// See [`millennium_utils::platform::current_exe`] for possible security
/// implications.
///
/// # Examples
///
/// ```rust,no_run
/// use millennium::{api::process::current_binary, Env, Manager};
/// let current_binary_path = current_binary(&Env::default()).unwrap();
///
/// millennium::Builder::default()
///   .setup(|app| {
///     let current_binary_path = current_binary(&app.env()).unwrap();
///     Ok(())
///   });
/// ```
///
/// [AppImage]: https://appimage.org/
pub fn current_binary(_env: &Env) -> std::io::Result<PathBuf> {
	// if we are running from an AppImage, we ONLY want the set AppImage path
	#[cfg(target_os = "linux")]
	if let Some(app_image_path) = &_env.appimage {
		return Ok(PathBuf::from(app_image_path));
	}

	millennium_utils::platform::current_exe()
}

/// Restarts the currently running binary.
///
/// See [`current_binary`] for platform specific behavior, and
/// [`millennium_utils::platform::current_exe`] for possible security
/// implications.
///
/// # Examples
///
/// ```rust,no_run
/// use millennium::{api::process::restart, Env, Manager};
///
/// millennium::Builder::default()
///   .setup(|app| {
///     restart(&app.env());
///     Ok(())
///   });
/// ```
pub fn restart(env: &Env) {
	use std::process::{exit, Command};

	if let Ok(path) = current_binary(env) {
		Command::new(path).spawn().expect("application failed to start");
	}

	exit(0);
}
