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

//! Types and functions related to file operations.

#[cfg(feature = "fs-extract-api")]
mod extract;
mod file_move;

use std::{fs, path::Path};

#[cfg(feature = "fs-extract-api")]
pub use extract::*;
pub use file_move::*;

/// Reads the entire contents of a file into a string.
pub fn read_string<P: AsRef<Path>>(file: P) -> crate::api::Result<String> {
	fs::read_to_string(file).map_err(Into::into)
}

/// Reads the entire contents of a file into a bytes vector.
pub fn read_binary<P: AsRef<Path>>(file: P) -> crate::api::Result<Vec<u8>> {
	fs::read(file).map_err(Into::into)
}

#[cfg(test)]
mod test {
	use super::*;
	#[allow(unused_imports)]
	use crate::api::Error;

	#[test]
	fn check_read_string() {
		let file = String::from("test/api/test.txt");

		let res = read_string(file);

		assert!(res.is_ok());

		if let Ok(s) = res {
			assert_eq!(s, "This is a test doc!\n".to_string());
		}
	}

	#[test]
	fn check_read_string_fail() {
		let file = String::from("test/api/");

		let res = read_string(file);

		assert!(res.is_err());

		#[cfg(not(windows))]
		if let Error::Io(e) = res.unwrap_err() {
			#[cfg(not(windows))]
			assert_eq!(e.to_string(), "Is a directory (os error 21)".to_string());
		}
	}

	#[test]
	fn check_read_binary() {
		let file = String::from("test/api/test_binary");

		let expected_vec = vec![
			35, 33, 47, 98, 105, 110, 47, 98, 97, 115, 104, 10, 10, 101, 99, 104, 111, 32, 34, 72, 101, 108, 108, 111, 32, 116, 104, 101, 114, 101, 34, 10,
		];

		let res = read_binary(file);

		assert!(res.is_ok());

		if let Ok(vec) = res {
			assert_eq!(vec, expected_vec);
		}
	}

	#[test]
	fn check_read_binary_fail() {
		let file = String::from("test/api/");

		let res = read_binary(file);

		assert!(res.is_err());

		#[cfg(not(windows))]
		if let Error::Io(e) = res.unwrap_err() {
			#[cfg(not(windows))]
			assert_eq!(e.to_string(), "Is a directory (os error 21)".to_string());
		}
	}
}
