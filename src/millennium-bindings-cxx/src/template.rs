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
extern crate url;

use std::{
	mem, ptr,
	sync::{Arc, Mutex}
};

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
		null_pointer_check!($ptr, crate::Nullable::NULL)
	};
	($ptr:expr, $null:expr) => {{
		#[allow(unused_imports)]
		if <_ as crate::Nullable>::is_null(&$ptr) {
			// TODO: real error handling
			panic!("null pointer");
		}
	}};
}

/// A `null` pointer was encountered where it wasn't expected.
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[error("A null pointer was encountered where it wasn't expected")]
pub struct NullPointer;

/////////////////////////////// end MIT code ///////////////////////////////

struct OnDrop<F: FnOnce()>(mem::ManuallyDrop<F>);

impl<F: FnOnce()> Drop for OnDrop<F> {
	#[inline(always)]
	fn drop(&mut self) {
		(unsafe { ptr::read(&*self.0) })();
	}
}

#[inline(always)]
fn on_unwind<F: FnOnce() -> T, T, P: FnOnce()>(f: F, p: P) -> T {
	let x = OnDrop(mem::ManuallyDrop::new(p));
	let t = f();
	let mut x = mem::ManuallyDrop::new(x);
	unsafe { mem::ManuallyDrop::drop(&mut x.0) };
	t
}

/// Temporarily takes ownership of a value at a mutable location, and replace it with a new value
/// based on the old one. Aborts on panic.
///
/// We move out of the reference temporarily, to apply a closure `f`, returning a new value, which
/// is then placed at the original value's location.
///
/// # Safety
///
/// It is crucial to only ever use this function having defined `panic = "abort"`, or else bad
/// things may happen.
#[inline]
pub fn replace_with<T, V, F>(v: *mut V, f: F)
where
	F: FnOnce(T) -> T
{
	let v = v as *mut T;

	unsafe {
		let old = ptr::read(v);
		let new = on_unwind(
			move || f(old),
			#[allow(clippy::redundant_closure)]
			|| std::process::abort()
		);
		ptr::write(v, new);
	};
}

#[repr(C)]
struct OpaqueContainer(Arc<Mutex<Option<*mut std::ffi::c_void>>>);

impl OpaqueContainer {
	pub fn get(&self) -> *mut std::ffi::c_void {
		let mut lock = self.0.lock().unwrap();
		let ptr = lock.as_mut().take().unwrap();
		*ptr
	}
}

unsafe impl Send for OpaqueContainer {}
unsafe impl Sync for OpaqueContainer {}

mod millennium_builder {
	use std::os::raw::*;
	use std::sync::{Arc, Mutex};

	#[repr(C)]
	pub struct BuilderFFI(());

	#[no_mangle]
	pub extern "C" fn millennium_builder_new() -> *mut BuilderFFI {
		let builder = millennium::Builder::default();
		Box::into_raw(Box::new(builder)) as _
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_builder_run(builder_ptr: *mut BuilderFFI) {
		null_pointer_check!(builder_ptr);

		let builder = builder_ptr as *mut millennium::Builder<millennium::MillenniumWebview>;
		let builder = builder.read();
		builder.run(millennium::generate_context!("$rc_path")).expect("error running application");
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_builder_setup(
		builder_ptr: *mut BuilderFFI,
		callback: unsafe extern "C" fn(*mut c_void, *mut millennium::App),
		opaque: *mut c_void
	) {
		null_pointer_check!(builder_ptr);

		let opaque = super::OpaqueContainer(Arc::new(Mutex::new(Some(opaque))));
		super::replace_with::<millennium::Builder<millennium::MillenniumWebview>, _, _>(builder_ptr, |builder| {
			builder.setup(move |app| {
				callback(opaque.get(), app);
				Ok(())
			})
		});
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_builder_invoke_handler(
		builder_ptr: *mut BuilderFFI,
		callback: unsafe extern "C" fn(*mut c_void, *mut super::millennium_invoke::MillenniumInvoke),
		opaque: *mut c_void
	) {
		null_pointer_check!(builder_ptr);

		let opaque = super::OpaqueContainer(Arc::new(Mutex::new(Some(opaque))));
		super::replace_with::<millennium::Builder<millennium::MillenniumWebview>, _, _>(builder_ptr, |builder| {
			builder.invoke_handler(move |invoke| {
				let mut invoke = super::millennium_invoke::MillenniumInvoke {
					message: Box::into_raw(Box::new(invoke.message)),
					resolver: Box::into_raw(Box::new(invoke.resolver))
				};

				callback(opaque.get(), &mut invoke);
			})
		});
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_builder_free(builder_ptr: *mut BuilderFFI) {
		null_pointer_check!(builder_ptr);

		let builder = builder_ptr as *mut millennium::Builder<millennium::MillenniumWebview>;
		let builder = builder.read();
		drop(builder);
		Box::from_raw(builder_ptr);
	}
}

mod millennium_window_builder {
	use std::ffi::CStr;
	use std::os::raw::*;

	#[repr(C)]
	pub struct WindowBuilderFFI(());

	#[no_mangle]
	pub unsafe extern "C" fn millennium_window_builder_new(
		app_ptr: *mut millennium::App,
		label: *const c_char,
		url: *const c_char
	) -> *mut WindowBuilderFFI {
		null_pointer_check!(app_ptr);

		let builder = millennium::window::WindowBuilder::new(
			&*app_ptr,
			CStr::from_ptr(label).to_str().expect("error converting label to &str"),
			millennium::utils::config::WindowUrl::External(url::Url::parse(CStr::from_ptr(url).to_str().expect("error converting url to &str")).expect("error parsing url"))
		);
		Box::into_raw(Box::new(builder)) as _
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_window_builder_title(
		builder_ptr: *mut WindowBuilderFFI,
		title: *const c_char
	) {
		null_pointer_check!(builder_ptr);

		super::replace_with::<millennium::window::WindowBuilder<millennium::MillenniumWebview>, _, _>(builder_ptr, |builder| {
			builder.title(CStr::from_ptr(title).to_str().expect("error converting title to &str"))
		});
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_window_builder_center(builder_ptr: *mut WindowBuilderFFI) {
		null_pointer_check!(builder_ptr);

		super::replace_with::<millennium::window::WindowBuilder<millennium::MillenniumWebview>, _, _>(builder_ptr, |builder| {
			builder.center()
		});
	}

	#[no_mangle]
	pub unsafe extern "C" fn millennium_window_builder_build(
		window_builder_ptr: *mut WindowBuilderFFI
	) {
		null_pointer_check!(window_builder_ptr);

		let window_builder = window_builder_ptr as *mut millennium::window::WindowBuilder<millennium::MillenniumWebview>;
		let window_builder = window_builder.read();
		window_builder.build().expect("error building window");
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
	pub extern "C" fn millennium_invoke_message_command(invoke_message_ptr: *mut millennium::InvokeMessage<millennium::MillenniumWebview>) -> *const c_char {
		let invoke_message = unsafe { Box::from_raw(invoke_message_ptr) };
		let command = invoke_message.command();
		let command_cstring = CString::new(command).expect("CString failed in millennium_invoke_message_command");
		command_cstring.into_raw()
	}

	#[no_mangle]
	pub extern "C" fn millennium_invoke_message_window(
		invoke_message_ptr: *mut millennium::InvokeMessage<millennium::MillenniumWebview>
	) -> *mut millennium::window::Window<millennium::MillenniumWebview> {
		let invoke_message = unsafe { Box::from_raw(invoke_message_ptr) };
		let window = invoke_message.window();
		Box::into_raw(Box::new(window))
	}

	#[no_mangle]
	pub extern "C" fn millennium_invoke_message_payload(invoke_message_ptr: *mut millennium::InvokeMessage<millennium::MillenniumWebview>) -> *const c_char {
		let invoke_message = unsafe { Box::from_raw(invoke_message_ptr) };
		let payload = invoke_message.payload();
		let payload = serde_json::to_string(&payload).unwrap();
		let payload_cstring = CString::new(payload).expect("CString failed in millennium_invoke_message_payload");
		payload_cstring.into_raw()
	}
}

mod millennium_window {
	use std::os::raw::*;

	#[no_mangle]
	pub extern "C" fn millennium_window_label(window_ptr: *mut millennium::window::Window<millennium::MillenniumWebview>) -> *const c_char {
		let window = unsafe { Box::from_raw(window_ptr) };
		let label = window.label();
		let label_ptr = Box::into_raw(Box::new(label));
		label_ptr as *const c_char
	}
}
