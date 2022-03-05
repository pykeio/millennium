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

#![cfg(target_os = "android")]

use ndk::configuration::Configuration;
use ndk_glue::Rect;

use crate::{
	event_loop::{EventLoop, EventLoopWindowTarget},
	window::{Window, WindowBuilder}
};

/// Additional methods on `EventLoop` that are specific to Android.
pub trait EventLoopExtAndroid {}

impl<T> EventLoopExtAndroid for EventLoop<T> {}

/// Additional methods on `EventLoopWindowTarget` that are specific to Android.
pub trait EventLoopWindowTargetExtAndroid {}

/// Additional methods on `Window` that are specific to Android.
pub trait WindowExtAndroid {
	fn content_rect(&self) -> Rect;

	fn config(&self) -> Configuration;
}

impl WindowExtAndroid for Window {
	fn content_rect(&self) -> Rect {
		self.window.content_rect()
	}

	fn config(&self) -> Configuration {
		self.window.config()
	}
}

impl<T> EventLoopWindowTargetExtAndroid for EventLoopWindowTarget<T> {}

/// Additional methods on `WindowBuilder` that are specific to Android.
pub trait WindowBuilderExtAndroid {}

impl WindowBuilderExtAndroid for WindowBuilder {}
