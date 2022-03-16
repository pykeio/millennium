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

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, DeriveInput, GenericParam, Ident, Token, Type, TypeParam};

/// The default runtime type to enable when the provided feature is enabled.
pub(crate) struct Attributes {
	default_type: Type,
	feature: Ident
}

impl Parse for Attributes {
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		let default_type = input.parse()?;
		input.parse::<Token![,]>()?;
		Ok(Attributes {
			default_type,
			feature: input.parse()?
		})
	}
}

pub(crate) fn default_runtime(attributes: Attributes, input: DeriveInput) -> TokenStream {
	// create a new copy to manipulate for the Millennium Webview feature flag
	let mut webview = input.clone();
	let webview_runtime = webview
		.generics
		.params
		.last_mut()
		.expect("default_runtime requires the item to have at least 1 generic parameter");

	// set the default value of the last generic parameter to the provided runtime
	// type
	match webview_runtime {
		GenericParam::Type(param @ TypeParam { eq_token: None, default: None, .. }) => {
			param.eq_token = Some(parse_quote!(=));
			param.default = Some(attributes.default_type);
		}
		_ => {
			panic!("DefaultRuntime requires the last parameter to not have a default value")
		}
	};

	let feature = attributes.feature.to_string();

	quote!(
		#[cfg(feature = #feature)]
		#webview

		#[cfg(not(feature = #feature))]
		#input
	)
}
