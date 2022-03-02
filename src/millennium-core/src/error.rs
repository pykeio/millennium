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

//! The `Error` struct and associated types.
use std::{error, fmt};

use crate::platform_impl;

/// An error whose cause it outside Millennium Core's control.
#[non_exhaustive]
#[derive(Debug)]
pub enum ExternalError {
	/// The operation is not supported by the backend.
	NotSupported(NotSupportedError),
	/// The OS cannot perform the operation.
	Os(OsError)
}

/// The error type for when the requested operation is not supported by the
/// backend.
#[derive(Clone)]
pub struct NotSupportedError {
	_marker: ()
}

/// The error type for when the OS cannot perform the requested operation.
#[derive(Debug)]
pub struct OsError {
	line: u32,
	file: &'static str,
	error: platform_impl::OsError
}

impl NotSupportedError {
	#[inline]
	#[allow(dead_code)]
	pub(crate) fn new() -> NotSupportedError {
		NotSupportedError { _marker: () }
	}
}

impl OsError {
	#[allow(dead_code)]
	pub(crate) fn new(line: u32, file: &'static str, error: platform_impl::OsError) -> OsError {
		OsError { line, file, error }
	}
}

#[allow(unused_macros)]
macro_rules! os_error {
	($error:expr) => {{
		crate::error::OsError::new(line!(), file!(), $error)
	}};
}

impl fmt::Display for OsError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		f.pad(&format!("os error at {}:{}: {}", self.file, self.line, self.error))
	}
}

impl fmt::Display for ExternalError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		match self {
			ExternalError::NotSupported(e) => e.fmt(f),
			ExternalError::Os(e) => e.fmt(f)
		}
	}
}

impl fmt::Debug for NotSupportedError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		f.debug_struct("NotSupportedError").finish()
	}
}

impl fmt::Display for NotSupportedError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		f.pad("the requested operation is not supported")
	}
}

impl error::Error for OsError {}
impl error::Error for ExternalError {}
impl error::Error for NotSupportedError {}
