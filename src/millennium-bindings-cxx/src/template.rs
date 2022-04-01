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

extern crate millennium;
extern crate serde_json;

use thiserror::Error;

// The following section contains code from `ffi-helpers`: https://github.com/Michael-F-Bryan/ffi_helpers
// Licensed under the MIT license.
//
// MIT License
//
// Copyright (c) 2018 Michael Bryan
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/// An object which has an "obviously invalid" value, for use with the
/// `null_pointer_check!()` macro.
///
/// This trait is implemented for all integer types and raw pointers, returning
/// `0` and `null` respectively.
pub trait Nullable {
	const NULL: Self;

	fn is_null(&self) -> bool;
}

macro_rules! impl_nullable_integer {
	($first: ty, $($rest: ty),* $(,)?) => {
		impl Nullable for $first {
			const NULL: Self = 0;

			#[inline]
			fn is_null(&self) -> bool { *self == Self::NULL }
		}

		impl_nullable_integer!($($rest,)*);
	};
	() => {};
}

impl<T> Nullable for *const T {
	const NULL: Self = std::ptr::null();

	#[inline]
	fn is_null(&self) -> bool {
		*self == Self::NULL
	}
}

impl<T> Nullable for *mut T {
	const NULL: Self = std::ptr::null_mut();

	#[inline]
	fn is_null(&self) -> bool {
		*self == Self::NULL
	}
}

impl_nullable_integer!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

impl<T> Nullable for Option<T> {
	const NULL: Self = None;

	#[inline]
	fn is_null(&self) -> bool {
		self.is_none()
	}
}

impl Nullable for () {
	const NULL: Self = ();

	#[inline]
	fn is_null(&self) -> bool {
		true
	}
}

/// Check if we've been given a null pointer, if so we'll return early.
///
/// The returned value is the [`NULL`] value for whatever type the calling
/// function returns. The `LAST_ERROR` thread-local variable is also updated
/// with [`NullPointer`].
#[macro_export]
macro_rules! null_pointer_check {
	($ptr:expr) => {
		null_pointer_check!($ptr, Nullable::NULL)
	};
	($ptr:expr, $null:expr) => {{
		#[allow(unused_imports)]
		if <_ as Nullable>::is_null(&$ptr) {
			update_last_error(NullPointer);
			return $null;
		}
	}};
}

/// A `null` pointer was encountered where it wasn't expected.
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[error("A null pointer was encountered where it wasn't expected")]
pub struct NullPointer;

/////////////////////////////// end MIT code ///////////////////////////////

mod millennium_builder {
	use std::os::raw::*;

	#[no_mangle]
	pub extern "C" fn millennium_builder_new() -> *mut millennium::Builder<millennium::MillenniumWebview> {
		let builder = millennium::Builder::default();
		let builder_ptr = Box::into_raw(Box::new(builder));
		builder_ptr
	}

	#[no_mangle]
	pub extern "C" fn millennium_builder_run(builder_ptr: *mut millennium::Builder<millennium::MillenniumWebview>) -> *mut c_void {
		let builder = unsafe { Box::from_raw(builder_ptr) };
		let result = builder.run(millennium::generate_context!("$rc_path"));
		let result_ptr = Box::into_raw(Box::new(result));
		result_ptr as *mut c_void
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_builder_setup(
		builder_ptr: *mut millennium::Builder<millennium::MillenniumWebview>,
		callback: unsafe extern "C" fn(*mut millennium::App)
	) -> *mut millennium::Builder<millennium::MillenniumWebview> {
		let builder = Box::from_raw(builder_ptr);
		*builder_ptr = builder.setup(move |app| {
			callback(app);
			Ok(())
		});
		builder_ptr
	}
}

mod millennium_invoke {
	use std::ffi::CString;
	use std::os::raw::*;

	#[repr(C)]
	pub struct MillenniumInvoke {
		pub message: *mut millennium::InvokeMessage<millennium::MillenniumWebview>,
		pub resolver: *mut millennium::InvokeResolver<millennium::MillenniumWebview>
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_invoke_message_command(
		invoke_message_ptr: *mut millennium::InvokeMessage<millennium::MillenniumWebview>
	) -> *const c_char {
		let invoke_message = Box::from_raw(invoke_message_ptr);
		let command = invoke_message.command();
		let command_cstring = CString::new(command).expect("CString failed in millennium_invoke_message_command");
		command_cstring.into_raw()
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_invoke_message_window(
		invoke_message_ptr: *mut millennium::InvokeMessage<millennium::MillenniumWebview>
	) -> *mut millennium::window::Window<millennium::MillenniumWebview> {
		let invoke_message = Box::from_raw(invoke_message_ptr);
		let window = invoke_message.window();
		let window_ptr = Box::into_raw(Box::new(window));
		window_ptr
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_invoke_message_payload(
		invoke_message_ptr: *mut millennium::InvokeMessage<millennium::MillenniumWebview>
	) -> *const c_char {
		let invoke_message = Box::from_raw(invoke_message_ptr);
		let payload = invoke_message.payload();
		let payload = serde_json::to_string(&payload).unwrap();
		let payload_cstring = CString::new(payload).expect("CString failed in millennium_invoke_message_payload");
		payload_cstring.into_raw()
	}
}

mod millennium_window {
	use std::os::raw::*;

	#[no_mangle]
	pub unsafe extern "C" fn millennium_window_label(window_ptr: *mut millennium::window::Window<millennium::MillenniumWebview>) -> *const c_char {
		let window = Box::from_raw(window_ptr);
		let label = window.label();
		let label_ptr = Box::into_raw(Box::new(label));
		label_ptr as *const c_char
	}
}