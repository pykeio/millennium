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

use std::marker::PhantomData;
#[cfg(feature = "isolation")]
use std::sync::Arc;

use millennium_utils::assets::{Assets, EmbeddedAssets};
use serde::Serialize;
use serialize_to_javascript::{default_template, Template};

/// An application pattern.
#[derive(Debug, Clone)]
pub enum Pattern<A: Assets = EmbeddedAssets> {
	/// The brownfield pattern.
	Brownfield(PhantomData<A>),
	/// Isolation pattern. Recommended for security purposes.
	#[cfg(feature = "isolation")]
	Isolation {
		/// The HTML served on `isolation://index.html`.
		assets: Arc<A>,

		/// The schema used for the isolation frames.
		schema: String,

		/// A random string used to ensure that the message went through the
		/// isolation frame.
		///
		/// This should be regenerated at runtime.
		key: String,

		/// Cryptographically secure keys
		crypto_keys: Box<millennium_utils::pattern::isolation::Keys>
	}
}

/// The shape of the JavaScript Pattern config
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase", tag = "pattern")]
pub(crate) enum PatternObject {
	/// Brownfield pattern.
	Brownfield,
	/// Isolation pattern. Recommended for security purposes.
	#[cfg(feature = "isolation")]
	Isolation {
		/// Which `IsolationSide` this `PatternObject` is getting injected into
		side: IsolationSide
	}
}

impl From<&Pattern> for PatternObject {
	fn from(pattern: &Pattern) -> Self {
		match pattern {
			Pattern::Brownfield(_) => Self::Brownfield,
			#[cfg(feature = "isolation")]
			Pattern::Isolation { .. } => Self::Isolation {
				side: IsolationSide::default()
			}
		}
	}
}

/// Where the JavaScript is injected to
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum IsolationSide {
	/// Original frame, the Brownfield application
	Original,
	/// Secure frame, the isolation security application
	#[allow(dead_code)]
	Secure
}

impl Default for IsolationSide {
	fn default() -> Self {
		Self::Original
	}
}

#[derive(Template)]
#[default_template("../scripts/pattern.js")]
pub(crate) struct PatternJavascript {
	pub(crate) pattern: PatternObject
}

#[allow(dead_code)]
pub(crate) fn format_real_schema(schema: &str) -> String {
	if cfg!(windows) {
		format!("https://{}.localhost", schema)
	} else {
		format!("{}://localhost", schema)
	}
}
