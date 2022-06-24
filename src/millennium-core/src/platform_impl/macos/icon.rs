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

use std::io::Cursor;

use crate::icon::{BadIcon, RgbaIcon};

#[derive(Debug, Clone)]
pub struct PlatformIcon(RgbaIcon);

impl PlatformIcon {
	pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
		Ok(PlatformIcon(RgbaIcon::from_rgba(rgba, width, height)?))
	}

	pub fn to_png(&self) -> Vec<u8> {
		let mut png = Vec::new();

		{
			let mut encoder = png::Encoder::new(Cursor::new(&mut png), self.0.width as _, self.0.height as _);
			encoder.set_color(png::ColorType::Rgba);
			encoder.set_depth(png::BitDepth::Eight);

			let mut writer = encoder.write_header().unwrap();
			writer.write_image_data(&self.0.rgba).unwrap();
		}

		png
	}
}
