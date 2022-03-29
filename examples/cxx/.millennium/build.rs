// Copyright 2022 pyke.io
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

//! This script uses millennium-bindings-cxx to generate C++ bindings for Millennium based on patterns analyzed from
//! your C++ code. You shouldn't have to modify any of the logic in here, Millennium will do that for you. Just modify
//! the settings below to match your project setup.
//!
//! The generated C/C++ header file is included in the `target` folder relative to the `build.rs` file. The generated
//! libraries (both static and shared) are included in the `target/{config}` folder relative to the `build.rs` file,
//! where {config} is the Rust target configuration (`debug` or `release`).
//!
//! This crate can be cross-compiled:
//! ```bash
//! $ rustup target add x86_64-unknown-linux-gnu
//! $ rustup toolchain install stable-x86_64-unknown-linux-gnu
//! $ cargo build --target x86_64-unknown-linux-gnu
//! ```
//!
//! To build a release build (with size optimizations), run `cargo build --release`.

/// The path to your .millenniumrc config file relative to the `build.rs` file. For most projects, this is
/// `../.millenniumrc`. Note that these bindings do **not** support platform-specific configuration files. If you need
/// conditional configuration for platforms, you can use something like the following:
///
/// ```rust
/// #[cfg(target_os = "macos")]
/// pub const MILLENNIUMRC_PATH: &str = "../.millenniumrc.macos";
/// #[cfg(target_os = "linux")]
/// pub const MILLENNIUMRC_PATH: &str = "../.millenniumrc.linux";
/// #[cfg(target_os = "windows")]
/// pub const MILLENNIUMRC_PATH: &str = "../.millenniumrc.windows";
/// ```
pub const MILLENNIUMRC_PATH: &str = "../.millenniumrc";
/// The path(s) to your C/C++ source code relative to the `build.rs` file.
/// This will be scanned by millennium-bindings-cxx to generate bindings.
/// In Rust, commands (among other things) are generated at compile time using Rust macros. Because we have to build the
/// Rust library before building C++, we have to scan the C++ beforehand to see what commands are being registered. The
/// C++ parsing is not at all perfect and is not context-aware, so be careful when using macro patterns and
/// use them with proper whitespace and don't use them in comments.
pub const CXX_SRC_PATHS: &[&str] = &["../src"];

extern crate millennium_bindings_cxx;

use std::{env, fs, path::PathBuf, time::Instant};

use millennium_build::{try_build, Attributes};

fn main() {
	// Always rebuild, even if nothing changed. This is so that millennium-bindings-cxx can re-scan C++ files
	// to detect new command patterns and generate bindings accordingly.
	env::set_var("MM_CXX_REBUILD", format!("{:?}", Instant::now()));
	println!("cargo:rerun-if-env-changed=MM_CXX_REBUILD");

	millennium_bindings_cxx::build(
		&PathBuf::from(
			// Far from ideal but it's the only way to get the path to the `build.rs` file.
			PathBuf::from(env::var("OUT_DIR").unwrap())
				.parent() // target/debug/build/.../
				.unwrap()
				.parent() // target/debug/build/
				.unwrap()
				.parent() // target/debug/
				.unwrap()
				.parent() // target/
				.unwrap()
		),
		MILLENNIUMRC_PATH
	)
	.expect("failed to build cxx bindings");

	env::set_var("MILLENNIUM_CONFIG", fs::read_to_string(MILLENNIUMRC_PATH).unwrap());
	if let Err(error) = try_build(Attributes::new()) {
		panic!("error during millennium-build: {:#?}", error);
	}
}
