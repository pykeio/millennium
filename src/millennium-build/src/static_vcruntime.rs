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

// taken from https://github.com/ChrisDenton/static_vcruntime/
// we're not using static_vcruntime directly because we want this for debug builds too

#![allow(dead_code)]

use std::{env, fs, io::Write, path::Path};

pub fn build() {
	override_msvcrt_lib();

	// Disable conflicting libraries that aren't hardcoded by Rust.
	println!("cargo:rustc-link-arg=/NODEFAULTLIB:libvcruntimed.lib");
	println!("cargo:rustc-link-arg=/NODEFAULTLIB:vcruntime.lib");
	println!("cargo:rustc-link-arg=/NODEFAULTLIB:vcruntimed.lib");
	println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmtd.lib");
	println!("cargo:rustc-link-arg=/NODEFAULTLIB:msvcrt.lib");
	println!("cargo:rustc-link-arg=/NODEFAULTLIB:msvcrtd.lib");
	println!("cargo:rustc-link-arg=/NODEFAULTLIB:libucrt.lib");
	println!("cargo:rustc-link-arg=/NODEFAULTLIB:libucrtd.lib");
	// Set the libraries we want.
	println!("cargo:rustc-link-arg=/DEFAULTLIB:libcmt.lib");
	println!("cargo:rustc-link-arg=/DEFAULTLIB:libvcruntime.lib");
	println!("cargo:rustc-link-arg=/DEFAULTLIB:ucrt.lib");
}

/// Override the hard-coded msvcrt.lib by replacing it with a (mostly) empty object file.
fn override_msvcrt_lib() {
	// Get the right machine type for the empty library.
	let arch = std::env::var("CARGO_CFG_TARGET_ARCH");
	let machine: &[u8] = if arch.as_deref() == Ok("x86_64") {
		&[0x64, 0x86]
	} else if arch.as_deref() == Ok("x86") {
		&[0x4C, 0x01]
	} else {
		return;
	};
	let bytes: &[u8] = &[
		1, 0, 94, 3, 96, 98, 60, 0, 0, 0, 1, 0, 0, 0, 0, 0, 132, 1, 46, 100, 114, 101, 99, 116, 118, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 16, 0, 46, 100, 114, 101, 99, 116, 118, 101, 0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 4, 0, 0, 0
	];

	// Write the empty "msvcrt.lib" to the output directory.
	let out_dir = env::var("OUT_DIR").unwrap();
	let path = Path::new(&out_dir).join("msvcrt.lib");
	let f = fs::OpenOptions::new().write(true).create_new(true).open(&path);
	if let Ok(mut f) = f {
		f.write_all(machine).unwrap();
		f.write_all(bytes).unwrap();
	}
	// Add the output directory to the native library path.
	println!("cargo:rustc-link-search=native={}", out_dir);
}
