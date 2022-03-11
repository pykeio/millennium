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

use std::array::TryFromSliceError;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::string::FromUtf8Error;

use aes_gcm::aead::Aead;
use aes_gcm::{aead::NewAead, Aes256Gcm, Nonce};
use once_cell::sync::OnceCell;
use serialize_to_javascript::{default_template, Template};

#[cfg(not(feature = "isolation"))]
mod ring_impl {
	#[cfg(not(feature = "__isolation-docs"))]
	compile_error!("Isolation RNG was used without enabling the `isolation` feature.");

	pub struct Unspecified;

	pub struct SystemRandom;

	impl SystemRandom {
		pub fn new() -> Self {
			unimplemented!()
		}
	}

	pub struct Random;

	impl Random {
		pub fn expose(self) -> [u8; 32] {
			unimplemented!()
		}
	}

	pub fn rand_generate(_rng: &SystemRandom) -> Result<Random, super::Error> {
		unimplemented!()
	}
}

#[cfg(feature = "isolation")]
mod ring_impl {
	pub use ring::error::Unspecified;
	pub use ring::rand::generate as rand_generate;
	pub use ring::rand::SystemRandom;
}

use ring_impl::*;

/// Cryptographically secure pseudo-random number generator.
static RNG: OnceCell<SystemRandom> = OnceCell::new();

/// The style for the isolation iframe.
pub const IFRAME_STYLE: &str = "#__millennium_isolation__ { display: none !important }";

/// Errors that can occur during Isolation keys generation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
	/// Something went wrong with the CSPRNG.
	#[error("Unspecified CSPRNG error")]
	Csprng,

	/// Something went wrong with decryping an AES-GCM payload
	#[error("AES-GCM")]
	Aes,

	/// Nonce was not 96 bits
	#[error("Nonce: {0}")]
	NonceSize(#[from] TryFromSliceError),

	/// Payload was not valid utf8
	#[error("{0}")]
	Utf8(#[from] FromUtf8Error),

	/// Invalid json format
	#[error("{0}")]
	Json(#[from] serde_json::Error)
}

impl From<Unspecified> for Error {
	fn from(_: Unspecified) -> Self {
		Self::Csprng
	}
}

/// A formatted AES-GCM cipher instance along with the key used to initialize
/// it.
#[derive(Clone)]
pub struct AesGcmPair {
	raw: [u8; 32],
	key: Aes256Gcm
}

impl Debug for AesGcmPair {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "AesGcmPair(...)")
	}
}

impl AesGcmPair {
	fn new() -> Result<Self, Error> {
		let rng = RNG.get_or_init(SystemRandom::new);
		let raw: [u8; 32] = ring_impl::rand_generate(rng)?.expose();
		let key = aes_gcm::Key::from_slice(&raw);
		Ok(Self { raw, key: Aes256Gcm::new(key) })
	}

	/// The raw value used to create the AES-GCM key
	pub fn raw(&self) -> &[u8; 32] {
		&self.raw
	}

	/// The formatted AES-GCM key
	pub fn key(&self) -> &Aes256Gcm {
		&self.key
	}
}

/// All cryptographic keys required for Isolation encryption
#[derive(Debug, Clone)]
pub struct Keys {
	/// AES-GCM key
	aes_gcm: AesGcmPair
}

impl Keys {
	/// Securely generate required keys for Isolation encryption.
	pub fn new() -> Result<Self, Error> {
		AesGcmPair::new().map(|aes_gcm| Self { aes_gcm }).map_err(Into::into)
	}

	/// The AES-GCM data (and raw data).
	pub fn aes_gcm(&self) -> &AesGcmPair {
		&self.aes_gcm
	}

	/// Decrypts a message using the generated keys.
	pub fn decrypt(&self, raw: RawIsolationPayload<'_>) -> Result<String, Error> {
		let RawIsolationPayload { nonce, payload } = raw;
		let nonce: [u8; 12] = nonce.as_ref().try_into()?;
		let bytes = self.aes_gcm.key.decrypt(Nonce::from_slice(&nonce), payload.as_ref()).map_err(|_| self::Error::Aes)?;

		String::from_utf8(bytes).map_err(Into::into)
	}
}

/// Raw representation of
#[derive(Debug, serde::Deserialize)]
pub struct RawIsolationPayload<'a> {
	nonce: Cow<'a, [u8]>,
	payload: Cow<'a, [u8]>
}

impl<'a> TryFrom<&'a str> for RawIsolationPayload<'a> {
	type Error = Error;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		serde_json::from_str(value).map_err(Into::into)
	}
}

/// The Isolation JavaScript template meant to be injected during codegen.
///
/// Note: This struct is not considered part of the stable API
#[derive(Template)]
#[default_template("isolation.js")]
pub struct IsolationJavascriptCodegen {
	// this template intentionally does not include the runtime field
}

/// The Isolation JavaScript template meant to be injected during runtime.
///
/// Note: This struct is not considered part of the stable API
#[derive(Template)]
#[default_template("isolation.js")]
pub struct IsolationJavascriptRuntime<'a> {
	/// The key used on the Rust backend and the Isolation Javascript
	pub runtime_aes_gcm_key: &'a [u8; 32]
}

#[cfg(test)]
mod test {
	#[test]
	fn create_keys() -> Result<(), Box<dyn std::error::Error>> {
		let _ = super::Keys::new()?;
		Ok(())
	}
}
