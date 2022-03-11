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

use std::{env::VarError, path::PathBuf};

use millennium_codegen::{context_codegen, get_config, ContextData};
use millennium_utils::config::parse::does_supported_extension_exist;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
	parse::{Parse, ParseBuffer},
	punctuated::Punctuated,
	LitStr, PathArguments, PathSegment, Token
};

pub(crate) struct ContextItems {
	config_file: PathBuf,
	root: syn::Path
}

impl Parse for ContextItems {
	fn parse(input: &ParseBuffer<'_>) -> syn::parse::Result<Self> {
		let config_file = if input.is_empty() {
			std::env::var("CARGO_MANIFEST_DIR").map(|m| PathBuf::from(m).join(".millenniumrc"))
		} else {
			let raw: LitStr = input.parse()?;
			let path = PathBuf::from(raw.value());
			if path.is_relative() {
				std::env::var("CARGO_MANIFEST_DIR").map(|m| PathBuf::from(m).join(path))
			} else {
				Ok(path)
			}
		}
		.map_err(|error| match error {
			VarError::NotPresent => "no CARGO_MANIFEST_DIR env var, this should be set by cargo".into(),
			VarError::NotUnicode(_) => "CARGO_MANIFEST_DIR env var contained invalid utf8".into()
		})
		.and_then(|path| {
			if does_supported_extension_exist(&path) {
				Ok(path)
			} else {
				Err(format!("no file at path {} exists, expected Millennium config file", path.display()))
			}
		})
		.map_err(|e| input.error(e))?;

		let context_path = if input.is_empty() {
			let mut segments = Punctuated::new();
			segments.push(PathSegment {
				ident: Ident::new("millennium", Span::call_site()),
				arguments: PathArguments::None
			});
			syn::Path {
				leading_colon: Some(Token![::](Span::call_site())),
				segments
			}
		} else {
			let _: Token![,] = input.parse()?;
			input.call(syn::Path::parse_mod_style)?
		};

		Ok(Self { config_file, root: context_path })
	}
}

pub(crate) fn generate_context(context: ContextItems) -> TokenStream {
	let context = get_config(&context.config_file)
		.map_err(|e| e.to_string())
		.map(|(config, config_parent)| ContextData {
			dev: cfg!(not(feature = "custom-protocol")),
			config,
			config_parent,
			root: context.root.to_token_stream()
		})
		.and_then(|data| context_codegen(data).map_err(|e| e.to_string()));

	match context {
		Ok(code) => code,
		Err(error) => quote!(compile_error!(#error))
	}
}
