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

use std::ffi::c_void;

use libc::*;

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
