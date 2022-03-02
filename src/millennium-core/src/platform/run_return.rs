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

#![cfg(not(target_os = "ios"))]

use crate::{
	event::Event,
	event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget}
};

/// Additional methods on `EventLoop` to return control flow to the caller.
pub trait EventLoopExtRunReturn {
	/// A type provided by the user that can be passed through
	/// `Event::UserEvent`.
	type UserEvent;

	/// Initializes the  event loop.
	///
	/// Unlike `run`, this function accepts non-`'static` (i.e. non-`move`)
	/// closures and returns control flow to the caller when `control_flow` is
	/// set to `ControlFlow::Exit`.
	///
	/// # Caveats
	/// Despite its appearance at first glance, this is *not* a perfect
	/// replacement for `poll_events`. For example, this function will not
	/// return on Windows or macOS while a window is getting resized, resulting
	/// in all application logic outside of the `event_handler` closure not
	/// running until the resize operation ends. Other OS operations
	/// may also result in such freezes. This behavior is caused by fundamental
	/// limitations in the underlying OS APIs, which cannot be hidden without
	/// severe stability repercussions.
	///
	/// You are strongly encouraged to use `run`, unless the use of this is
	/// absolutely necessary.
	///
	/// ## Platform-specific
	/// - **Unix-alikes** (**X11** or **Wayland**): This function returns `1`
	///   upon disconnection from the display server.
	fn run_return<F>(&mut self, event_handler: F) -> i32
	where
		F: FnMut(Event<'_, Self::UserEvent>, &EventLoopWindowTarget<Self::UserEvent>, &mut ControlFlow);
}

impl<T> EventLoopExtRunReturn for EventLoop<T> {
	type UserEvent = T;

	fn run_return<F>(&mut self, event_handler: F) -> i32
	where
		F: FnMut(Event<'_, Self::UserEvent>, &EventLoopWindowTarget<Self::UserEvent>, &mut ControlFlow)
	{
		self.event_loop.run_return(event_handler)
	}
}
