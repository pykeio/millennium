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

use thiserror::Error;

/// All errors that can occur while running the updater.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
	/// IO Errors.
	#[error("`{0}`")]
	Io(#[from] std::io::Error),
	/// Semver Errors.
	#[error("Unable to compare version: {0}")]
	Semver(#[from] semver::Error),
	/// JSON (Serde) Errors.
	#[error("JSON error: {0}")]
	SerdeJson(#[from] serde_json::Error),
	/// Minisign is used for signature validation.
	#[error("Verify signature error: {0}")]
	Minisign(#[from] minisign_verify::Error),
	/// Error with Minisign base64 decoding.
	#[error("Signature decoding error: {0}")]
	Base64(#[from] base64::DecodeError),
	/// UTF8 Errors in signature.
	#[error("Signature encoding error: {0}")]
	Utf8(#[from] std::str::Utf8Error),
	/// Millennium utils, mainly extract and file move.
	#[error("Millennium API error: {0}")]
	MillenniumApi(#[from] crate::api::Error),
	/// Network error.
	#[error("Network error: {0}")]
	Network(String),
	/// Metadata (JSON) error.
	#[error("Remote JSON error: {0}")]
	RemoteMetadata(String),
	/// Error building updater.
	#[error("Unable to prepare the updater: {0}")]
	Builder(String),
	/// Error building updater.
	#[error("Unable to extract the new version: {0}")]
	Extract(String),
	/// Updater is not supported for current operating system or platform.
	#[error("Unsuported operating system or platform")]
	UnsupportedPlatform,
	/// Public key found in `.millenniumrc` but no signature announced remotely.
	#[error("Signature not available, skipping update")]
	MissingUpdaterSignature,
	/// Triggered when there is NO error and the two versions are equals.
	/// On client side, it's important to catch this error.
	#[error("No updates available")]
	UpToDate
}

pub type Result<T = ()> = std::result::Result<T, Error>;
