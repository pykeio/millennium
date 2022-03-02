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

use quote::format_ident;
use syn::{
	parse::{Parse, ParseBuffer},
	Ident, Path, Token
};

/// The items parsed from [`generate_handle!`](crate::generate_handle).
pub struct Handler {
	paths: Vec<Path>,
	commands: Vec<Ident>,
	wrappers: Vec<Path>
}

impl Parse for Handler {
	fn parse(input: &ParseBuffer<'_>) -> syn::Result<Self> {
		let paths = input.parse_terminated::<Path, Token![,]>(Path::parse)?;

		// parse the command names and wrappers from the passed paths
		let (commands, wrappers) = paths
			.iter()
			.map(|path| {
				let mut wrapper = path.clone();
				let last = super::path_to_command(&mut wrapper);

				// the name of the actual command function
				let command = last.ident.clone();

				// set the path to the command function wrapper
				last.ident = super::format_command_wrapper(&command);

				(command, wrapper)
			})
			.unzip();

		Ok(Self {
			paths: paths.into_iter().collect(), // remove punctuation separators
			commands,
			wrappers
		})
	}
}

impl From<Handler> for proc_macro::TokenStream {
	fn from(Handler { paths, commands, wrappers }: Handler) -> Self {
		let cmd = format_ident!("__millennium_cmd__");
		let invoke = format_ident!("__millennium_invoke__");
		quote::quote!(move |#invoke| {
			let #cmd = #invoke.message.command();
			match #cmd {
				#(stringify!(#commands) => #wrappers!(#paths, #invoke),)*
				_ => {
					#invoke.resolver.reject(format!("command {} not found", #cmd))
				},
			}
		})
		.into()
	}
}
