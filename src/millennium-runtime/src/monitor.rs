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

use super::window::dpi::{PhysicalPosition, PhysicalSize};

/// Monitor descriptor.
#[derive(Debug, Clone)]
pub struct Monitor {
	/// A human-readable name of the monitor.
	/// `None` if the monitor doesn't exist anymore.
	pub name: Option<String>,
	/// The monitor's resolution.
	pub size: PhysicalSize<u32>,
	/// The top-left corner position of the monitor relative to the larger full
	/// screen area.
	pub position: PhysicalPosition<i32>,
	/// Returns the scale factor that can be used to map logical pixels to
	/// physical pixels, and vice versa.
	pub scale_factor: f64
}
