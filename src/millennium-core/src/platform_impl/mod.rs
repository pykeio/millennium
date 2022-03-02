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

pub use self::platform::*;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform;
#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
#[path = "linux/mod.rs"]
mod platform;
#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod platform;
#[cfg(target_os = "android")]
#[path = "android/mod.rs"]
mod platform;
#[cfg(target_os = "ios")]
#[path = "ios/mod.rs"]
mod platform;

#[cfg(all(
	not(target_os = "ios"),
	not(target_os = "windows"),
	not(target_os = "linux"),
	not(target_os = "macos"),
	not(target_os = "android"),
	not(target_os = "dragonfly"),
	not(target_os = "freebsd"),
	not(target_os = "netbsd"),
	not(target_os = "openbsd"),
))]
compile_error!("The platform you're compiling for is not supported!");
