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

use state::Container;

use crate::{
	command::{CommandArg, CommandItem},
	runtime::Runtime,
	InvokeError
};

/// A guard for a state value.
pub struct State<'r, T: Send + Sync + 'static>(&'r T);

impl<'r, T: Send + Sync + 'static> State<'r, T> {
	/// Retrieve a borrow to the underlying value with a lifetime of `'r`.
	/// Using this method is typically unnecessary as `State` implements
	/// [`std::ops::Deref`] with a [`std::ops::Deref::Target`] of `T`.
	#[inline(always)]
	pub fn inner(&self) -> &'r T {
		self.0
	}
}

impl<T: Send + Sync + 'static> std::ops::Deref for State<'_, T> {
	type Target = T;

	#[inline(always)]
	fn deref(&self) -> &T {
		self.0
	}
}

impl<T: Send + Sync + 'static> Clone for State<'_, T> {
	fn clone(&self) -> Self {
		State(self.0)
	}
}

impl<'r, 'de: 'r, T: Send + Sync + 'static, R: Runtime> CommandArg<'de, R> for State<'r, T> {
	/// Grabs the [`State`] from the [`CommandItem`]. This will never fail.
	fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
		Ok(command.message.state_ref().try_get().unwrap_or_else(|| {
			panic!(
				"state not managed for field `{}` on command `{}`. You muse call `.manage()` before using this command",
				command.key, command.name
			)
		}))
	}
}

/// The Millennium state manager.
#[derive(Debug)]
pub struct StateManager(pub(crate) Container![Send + Sync]);

impl StateManager {
	pub(crate) fn new() -> Self {
		Self(<Container![Send + Sync]>::new())
	}

	pub(crate) fn set<T: Send + Sync + 'static>(&self, state: T) -> bool {
		self.0.set(state)
	}

	/// Gets the state associated with the specified type.
	pub fn get<T: Send + Sync + 'static>(&self) -> State<'_, T> {
		State(self.0.try_get().expect("state: get() called before set() for given type"))
	}

	/// Gets the state associated with the specified type.
	pub fn try_get<T: Send + Sync + 'static>(&self) -> Option<State<'_, T>> {
		self.0.try_get().map(State)
	}
}
