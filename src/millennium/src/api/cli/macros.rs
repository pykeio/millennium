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

macro_rules! bind_string_arg {
	($arg:expr, $clap_arg:expr, $arg_name:ident, $clap_field:ident) => {{
		let arg = $arg;
		let mut clap_arg = $clap_arg;
		if let Some(value) = &arg.$arg_name {
			clap_arg = clap_arg.$clap_field(value.as_str());
		}
		clap_arg
	}};
}

macro_rules! bind_value_arg {
	($arg:expr, $clap_arg:expr, $field:ident) => {{
		let arg = $arg;
		let mut clap_arg = $clap_arg;
		if let Some(value) = arg.$field {
			clap_arg = clap_arg.$field(value);
		}
		clap_arg
	}};
}

macro_rules! bind_string_slice_arg {
	($arg:expr, $clap_arg:expr, $field:ident) => {{
		let arg = $arg;
		let mut clap_arg = $clap_arg;
		if let Some(value) = &arg.$field {
			let v: Vec<&str> = value.iter().map(|x| &**x).collect();
			clap_arg = clap_arg.$field(v);
		}
		clap_arg
	}};
}

macro_rules! bind_if_arg {
	($arg:expr, $clap_arg:expr, $field:ident) => {{
		let arg = $arg;
		let mut clap_arg = $clap_arg;
		if let Some(value) = &arg.$field {
			let v: Vec<&str> = value.iter().map(|x| &**x).collect();
			clap_arg = clap_arg.$field(&v[0], &v[1]);
		}
		clap_arg
	}};
}
