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

//! Contains traits with platform-specific methods in them.
//!
//! Contains the follow OS-specific modules:
//!
//!  - `android`
//!  - `ios`
//!  - `macos`
//!  - `unix`
//!  - `windows`
//!
//! And the following platform-specific module:
//!
//! - `global_shortcut` (available on `windows`, `unix`, `macos`)
//! - `run_return` (available on `windows`, `unix`, `macos`, and `android`)
//!
//! However only the module corresponding to the platform you're compiling to
//! will be available.

pub mod android;
pub mod ios;
pub mod macos;
pub mod run_return;
pub mod unix;
pub mod windows;
