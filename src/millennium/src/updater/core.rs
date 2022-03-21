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

#[cfg(feature = "updater")]
#[cfg(not(target_os = "macos"))]
use std::ffi::OsStr;
#[cfg(feature = "updater")]
use std::io::Seek;
use std::{
	collections::HashMap,
	env,
	io::{Cursor, Read},
	path::{Path, PathBuf},
	str::from_utf8
};
#[cfg(target_os = "windows")]
use std::{
	fs::read_dir,
	process::{exit, Command}
};

use base64::decode;
use futures::StreamExt;
use http::StatusCode;
use millennium_utils::{platform::current_exe, Env};
use minisign_verify::{PublicKey, Signature};

use super::error::{Error, Result};
#[cfg(all(feature = "updater", not(target_os = "windows")))]
use crate::api::file::Compression;
#[cfg(feature = "updater")]
use crate::api::file::{ArchiveFormat, Extract, Move};
use crate::{
	api::{
		http::{ClientBuilder, HttpRequestBuilder},
		version
	},
	AppHandle, Manager, Runtime
};

#[derive(Debug)]
pub struct RemoteRelease {
	/// Version to install
	pub version: String,
	/// Release date
	pub date: String,
	/// Download URL for current platform
	pub download_url: String,
	/// Update short description
	pub body: Option<String>,
	/// Optional signature for the current platform
	pub signature: Option<String>,
	#[cfg(target_os = "windows")]
	/// Optional: Windows only try to use elevated task
	pub with_elevated_task: bool
}

impl RemoteRelease {
	// Read JSON and confirm this is a valid Schema
	fn from_release(release: &serde_json::Value, target: &str) -> Result<RemoteRelease> {
		// Version or name is required for static and dynamic JSON
		// if `version` is not announced, we fallback to `name` (can be the tag name
		// example v1.0.0)
		let version = match release.get("version") {
			Some(version) => version
				.as_str()
				.ok_or_else(|| Error::RemoteMetadata("Unable to extract `version` from remote server".into()))?
				.trim_start_matches('v')
				.to_string(),
			None => release
				.get("name")
				.ok_or_else(|| Error::RemoteMetadata("Release missing `name` and `version`".into()))?
				.as_str()
				.ok_or_else(|| Error::RemoteMetadata("Unable to extract `name` from remote server`".into()))?
				.trim_start_matches('v')
				.to_string()
		};

		// pub_date is required default is: `N/A` if not provided by the remote JSON
		let date = release.get("pub_date").and_then(|v| v.as_str()).unwrap_or("N/A").to_string();

		// body is optional to build our update
		let body = release.get("notes").map(|notes| notes.as_str().unwrap_or("").to_string());

		// signature is optional to build our update
		let mut signature = release.get("signature").map(|signature| signature.as_str().unwrap_or("").to_string());

		let download_url;
		#[cfg(target_os = "windows")]
		let with_elevated_task;

		match release.get("platforms") {
			// Did we have a platforms field?
			// If we did, that mean it's a static JSON.
			// The main difference with STATIC and DYNAMIC is static announce ALL platforms
			// and dynamic announce only the current platform.
			//
			// This could be used if you do NOT want an update server and use
			// a GIST, S3 or any static JSON file to announce your updates.
			//
			// Notes:
			// Dynamic help to reduce bandwidth usage or to intelligently update your clients
			// based on the request you give. The server can remotely drive behaviors like
			// rolling back or phased rollouts.
			Some(platforms) => {
				// make sure we have our target available
				if let Some(current_target_data) = platforms.get(target) {
					// use provided signature if available
					signature = current_target_data
						.get("signature")
						.map(|found_signature| found_signature.as_str().unwrap_or("").to_string());
					// Download URL is required
					download_url = current_target_data
						.get("url")
						.ok_or_else(|| Error::RemoteMetadata("Release missing `url`".into()))?
						.as_str()
						.ok_or_else(|| Error::RemoteMetadata("Unable to extract `url` from remote server`".into()))?
						.to_string();
					#[cfg(target_os = "windows")]
					{
						with_elevated_task = current_target_data
							.get("with_elevated_task")
							.and_then(|v| v.as_bool())
							.unwrap_or_default();
					}
				} else {
					// make sure we have an available platform from the static
					return Err(Error::RemoteMetadata("Platform not available".into()));
				}
			}
			// We don't have the `platforms` field announced, let's assume our
			// download URL is at the root of the JSON.
			None => {
				download_url = release
					.get("url")
					.ok_or_else(|| Error::RemoteMetadata("Release missing `url`".into()))?
					.as_str()
					.ok_or_else(|| Error::RemoteMetadata("Unable to extract `url` from remote server`".into()))?
					.to_string();
				#[cfg(target_os = "windows")]
				{
					with_elevated_task = match release.get("with_elevated_task") {
						Some(with_elevated_task) => with_elevated_task.as_bool().unwrap_or(false),
						None => false
					};
				}
			}
		}
		// Return our formatted release
		Ok(RemoteRelease {
			version,
			date,
			download_url,
			body,
			signature,
			#[cfg(target_os = "windows")]
			with_elevated_task
		})
	}
}

#[derive(Debug)]
pub struct UpdateBuilder<'a, R: Runtime> {
	/// Application handle.
	pub app: AppHandle<R>,
	/// Current version we are running to compare with announced version
	pub current_version: &'a str,
	/// The URLs to checks updates. We suggest at least one fallback on a
	/// different domain.
	pub urls: Vec<String>,
	/// The platform the updater will check and install the update. Default is
	/// from `get_updater_target`
	pub target: Option<String>,
	/// The current executable path. Default is automatically extracted.
	pub executable_path: Option<PathBuf>
}

// Create new updater instance and return an Update
impl<'a, R: Runtime> UpdateBuilder<'a, R> {
	pub fn new(app: AppHandle<R>) -> Self {
		UpdateBuilder {
			app,
			urls: Vec::new(),
			target: None,
			executable_path: None,
			current_version: env!("CARGO_PKG_VERSION")
		}
	}

	#[allow(dead_code)]
	pub fn url(mut self, url: String) -> Self {
		self.urls
			.push(percent_encoding::percent_decode(url.as_bytes()).decode_utf8_lossy().to_string());
		self
	}

	/// Add multiple URLS at once inside a Vec for future reference
	pub fn urls(mut self, urls: &[String]) -> Self {
		let mut formatted_vec: Vec<String> = Vec::new();
		for url in urls {
			formatted_vec.push(percent_encoding::percent_decode(url.as_bytes()).decode_utf8_lossy().to_string());
		}
		self.urls = formatted_vec;
		self
	}

	/// Set the current app version, used to compare against the latest
	/// available version. The `cargo_crate_version!` macro can be used to pull
	/// the version from your `Cargo.toml`
	pub fn current_version(mut self, ver: &'a str) -> Self {
		self.current_version = ver;
		self
	}

	/// Set the target (os)
	/// win32, win64, darwin and linux are currently supported
	#[allow(dead_code)]
	pub fn target(mut self, target: &str) -> Self {
		self.target = Some(target.to_owned());
		self
	}

	/// Set the executable path
	#[allow(dead_code)]
	pub fn executable_path<A: AsRef<Path>>(mut self, executable_path: A) -> Self {
		self.executable_path = Some(PathBuf::from(executable_path.as_ref()));
		self
	}

	pub async fn build(self) -> Result<Update<R>> {
		let mut remote_release: Option<RemoteRelease> = None;

		// make sure we have at least one url
		if self.urls.is_empty() {
			return Err(Error::Builder("Unable to check update, `url` is required.".into()));
		};

		// set current version if not set
		let current_version = self.current_version;

		// If no executable path provided, we use current_exe from millennium_utils
		let executable_path = self.executable_path.unwrap_or(current_exe()?);

		// Did the target is provided by the config?
		// Should be: linux, darwin, win32 or win64
		let target = self.target.or_else(get_updater_target).ok_or(Error::UnsupportedPlatform)?;

		// Get the extract_path from the provided executable_path
		let extract_path = extract_path_from_executable(&self.app.state::<Env>(), &executable_path);

		// Set SSL certs for linux if they aren't available.
		// We do not require to recheck in the download_and_install as we use
		// ENV variables, we can expect them to be set for the second call.
		#[cfg(target_os = "linux")]
		{
			if env::var_os("SSL_CERT_FILE").is_none() {
				env::set_var("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
			}
			if env::var_os("SSL_CERT_DIR").is_none() {
				env::set_var("SSL_CERT_DIR", "/etc/ssl/certs");
			}
		}

		// Allow fallback if more than 1 urls is provided
		let mut last_error: Option<Error> = None;
		for url in &self.urls {
			// replace {{current_version}} and {{target}} in the provided URL
			// this is usefull if we need to query example
			// https://releases.myapp.com/update/{{target}}/{{current_version}}
			// will be transleted into ->
			// https://releases.myapp.com/update/darwin/1.0.0
			// The main objective is if the update URL is defined via the Cargo.toml
			// the URL will be generated dynamicly
			let fixed_link = str::replace(&str::replace(url, "{{current_version}}", current_version), "{{target}}", &target);

			// we want JSON only
			let mut headers = HashMap::new();
			headers.insert("Accept".into(), "application/json".into());

			let resp = ClientBuilder::new()
				.build()?
				.send(HttpRequestBuilder::new("GET", &fixed_link)?
						.headers(headers)
						// wait 20sec for the firewall
						.timeout(20))
				.await;

			// If we got a success, we stop the loop
			// and we set our remote_release variable
			if let Ok(res) = resp {
				let res = res.read().await?;
				// got status code 2XX
				if StatusCode::from_u16(res.status).map_err(|e| Error::Builder(e.to_string()))?.is_success() {
					// if we got 204
					if StatusCode::NO_CONTENT.as_u16() == res.status {
						// return with `UpToDate` error
						// we should catch on the client
						return Err(Error::UpToDate);
					};
					// Convert the remote result to our local struct
					let built_release = RemoteRelease::from_release(&res.data, &target);
					// make sure all went well and the remote data is compatible
					// with what we need locally
					match built_release {
						Ok(release) => {
							last_error = None;
							remote_release = Some(release);
							break;
						}
						Err(err) => last_error = Some(err)
					}
				} // if status code is not 2XX we keep loopin' our urls
			}
		}

		// Last error is cleaned on success -- shouldn't be triggered if
		// we have a successful call
		if let Some(error) = last_error {
			return Err(Error::Network(error.to_string()));
		}

		// Extracted remote metadata
		let final_release = remote_release.ok_or_else(|| Error::RemoteMetadata("Unable to extract update metadata from the remote server.".into()))?;

		// did the announced version is greated than our current one?
		let should_update = version::is_greater(current_version, &final_release.version).unwrap_or(false);

		// create our new updater
		Ok(Update {
			app: self.app,
			target,
			extract_path,
			should_update,
			version: final_release.version,
			date: final_release.date,
			current_version: self.current_version.to_owned(),
			download_url: final_release.download_url,
			body: final_release.body,
			signature: final_release.signature,
			#[cfg(target_os = "windows")]
			with_elevated_task: final_release.with_elevated_task
		})
	}
}

pub fn builder<'a, R: Runtime>(app: AppHandle<R>) -> UpdateBuilder<'a, R> {
	UpdateBuilder::new(app)
}

#[derive(Debug)]
pub struct Update<R: Runtime> {
	/// Application handle.
	pub app: AppHandle<R>,
	/// Update description
	pub body: Option<String>,
	/// Should we update or not
	pub should_update: bool,
	/// Version announced
	pub version: String,
	/// Running version
	pub current_version: String,
	/// Update publish date
	pub date: String,
	/// Target
	#[allow(dead_code)]
	target: String,
	/// Extract path
	extract_path: PathBuf,
	/// Download URL announced
	download_url: String,
	/// Signature announced
	signature: Option<String>,
	#[cfg(target_os = "windows")]
	/// Optional: Windows only try to use elevated task
	/// Default to false
	with_elevated_task: bool
}

impl<R: Runtime> Clone for Update<R> {
	fn clone(&self) -> Self {
		Update {
			app: self.app.clone(),
			body: self.body.clone(),
			should_update: self.should_update,
			version: self.version.clone(),
			current_version: self.current_version.clone(),
			date: self.date.clone(),
			target: self.target.clone(),
			extract_path: self.extract_path.clone(),
			download_url: self.download_url.clone(),
			signature: self.signature.clone(),
			#[cfg(target_os = "windows")]
			with_elevated_task: self.with_elevated_task
		}
	}
}

impl<R: Runtime> Update<R> {
	// Download and install our update
	// @todo(lemarier): Split into download and install (two step) but need to be
	// thread safe
	pub async fn download_and_install<F: Fn(usize, Option<u64>)>(&self, pub_key: String, on_chunk: F) -> Result {
		// make sure we can install the update on linux
		// We fail here because later we can add more linux support
		// actually if we use APPIMAGE, our extract path should already
		// be set with our APPIMAGE env variable, we don't need to do
		// anythin with it yet
		#[cfg(target_os = "linux")]
		if self.app.state::<Env>().appimage.is_none() {
			return Err(Error::UnsupportedPlatform);
		}

		// set our headers
		let mut headers = HashMap::new();
		headers.insert("Accept".into(), "application/octet-stream".into());
		headers.insert("User-Agent".into(), "millennium/updater".into());

		// Create our request
		let response = ClientBuilder::new()
			.build()?
			.send(HttpRequestBuilder::new("GET", self.download_url.as_str())?
					.headers(headers)
					// wait 20sec for the firewall
					.timeout(20))
			.await?;

		// make sure it's success
		if !response.status().is_success() {
			return Err(Error::Network(format!("Download request failed with status: {}", response.status())));
		}

		let content_length: Option<u64> = response
			.headers()
			.get("Content-Length")
			.and_then(|value| value.to_str().ok())
			.and_then(|value| value.parse().ok());

		let mut buffer = Vec::new();
		let mut stream = response.bytes_stream();
		while let Some(chunk) = stream.next().await {
			let chunk = chunk?;
			let bytes = chunk.as_ref().to_vec();
			on_chunk(bytes.len(), content_length);
			buffer.extend(bytes);
		}

		// create memory buffer from our archive (Seek + Read)
		let mut archive_buffer = Cursor::new(buffer);

		// We need an announced signature by the server
		// if there is no signature, bail out.
		if let Some(signature) = &self.signature {
			// we make sure the archive is valid and signed with the private key linked with
			// the publickey
			verify_signature(&mut archive_buffer, signature, &pub_key)?;
		} else {
			// We have a public key inside our source file, but not announced by the server,
			// we assume this update is NOT valid.
			return Err(Error::MissingUpdaterSignature);
		}

		#[cfg(feature = "updater")]
		{
			// we copy the files depending of the operating system
			// we run the setup, appimage re-install or overwrite the
			// macos .app
			#[cfg(target_os = "windows")]
			copy_files_and_run(archive_buffer, &self.extract_path, self.with_elevated_task)?;
			#[cfg(not(target_os = "windows"))]
			copy_files_and_run(archive_buffer, &self.extract_path)?;
		}
		// We are done!
		Ok(())
	}
}

// Linux (AppImage)

// ### Expected structure:
// ├── [AppName]_[version]_amd64.AppImage.tar.gz
// │   └──[AppName]_[version]_amd64.AppImage
// └── ...

// We should have an AppImage already installed to be able to copy and install
// the extract_path is the current AppImage path
// tmp_dir is where our new AppImage is found
#[cfg(feature = "updater")]
#[cfg(target_os = "linux")]
fn copy_files_and_run<R: Read + Seek>(archive_buffer: R, extract_path: &Path) -> Result {
	let tmp_dir = tempfile::Builder::new().prefix("millennium_current_app").tempdir()?;

	let tmp_app_image = &tmp_dir.path().join("current_app.AppImage");

	// create a backup of our current app image
	Move::from_source(extract_path).to_dest(tmp_app_image)?;

	// extract the buffer to the tmp_dir
	// we extract our signed archive into our final directory without any temp file
	let mut extractor = Extract::from_cursor(archive_buffer, ArchiveFormat::Tar(Some(Compression::Gz)));

	for file in extractor.files()? {
		if file.extension() == Some(OsStr::new("AppImage")) {
			// if something went wrong during the extraction, we should restore previous app
			if let Err(err) = extractor.extract_file(extract_path, &file) {
				Move::from_source(tmp_app_image).to_dest(extract_path)?;
				return Err(Error::Extract(err.to_string()));
			}
			// early finish we have everything we need here
			return Ok(());
		}
	}

	Ok(())
}

// Windows

// ### Expected structure:
// ├── [AppName]_[version]_x64.msi.zip
// │   └──[AppName]_[version]_x64.msi
// └── ...

// ## MSI
// Update server can provide a MSI for Windows. (Generated with
// millennium-bundler from *Wix*) To replace current version of the application.
// In later version we'll offer incremental update to push specific binaries.

// ## EXE
// Update server can provide a custom EXE (installer) who can run any task.
#[cfg(feature = "updater")]
#[cfg(target_os = "windows")]
#[allow(clippy::unnecessary_wraps)]
fn copy_files_and_run<R: Read + Seek>(archive_buffer: R, _extract_path: &Path, with_elevated_task: bool) -> Result {
	// FIXME: We need to create a memory buffer with the MSI and then run it.
	//        (instead of extracting the MSI to a temp path)
	//
	// The tricky part is the MSI need to be exposed and spawned so the memory
	// allocation shouldn't drop but we should be able to pass the reference so we
	// can drop it once the installation is done, otherwise we have a huge memory
	// leak.

	let tmp_dir = tempfile::Builder::new().tempdir()?.into_path();

	// extract the buffer to the tmp_dir
	// we extract our signed archive into our final directory without any temp file
	let mut extractor = Extract::from_cursor(archive_buffer, ArchiveFormat::Zip);

	// extract the msi
	extractor.extract_into(&tmp_dir)?;

	let paths = read_dir(&tmp_dir)?;
	// This consumes the TempDir without deleting directory on the filesystem,
	// meaning that the directory will no longer be automatically deleted.

	for path in paths {
		let found_path = path?.path();
		// we support 2 type of files exe & msi for now
		// If it's an `exe` we expect an installer not a runtime.
		if found_path.extension() == Some(OsStr::new("exe")) {
			// Run the EXE
			Command::new(found_path).spawn().expect("installer failed to start");

			exit(0);
		} else if found_path.extension() == Some(OsStr::new("msi")) {
			if with_elevated_task {
				if let Some(bin_name) = current_exe()
					.ok()
					.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
					.and_then(|s| s.into_string().ok())
				{
					let product_name = bin_name.replace(".exe", "");

					// Check if there is a task that enables the updater to skip the UAC prompt
					let update_task_name = format!("Update {} - Skip UAC", product_name);
					if let Ok(status) = Command::new("schtasks").arg("/QUERY").arg("/TN").arg(update_task_name.clone()).status() {
						if status.success() {
							// Rename the MSI to the match file name the Skip UAC task is expecting it to be
							let temp_msi = tmp_dir.with_file_name(bin_name).with_extension("msi");
							Move::from_source(&found_path).to_dest(&temp_msi).expect("Unable to move update MSI");
							let exit_status = Command::new("schtasks")
								.arg("/RUN")
								.arg("/TN")
								.arg(update_task_name)
								.status()
								.expect("failed to start updater task");

							if exit_status.success() {
								// Successfully launched task that skips the UAC prompt
								exit(0);
							}
						}
						// Failed to run update task. Following UAC Path
					}
				}
			}

			// restart should be handled by WIX as we exit the process
			Command::new("msiexec.exe")
				.arg("/i")
				.arg(found_path)
				// quiet basic UI with prompt at the end
				.arg("/qb+")
				.spawn()
				.expect("installer failed to start");

			exit(0);
		}
	}

	Ok(())
}

// MacOS
// ### Expected structure:
// ├── [AppName]_[version]_x64.app.tar.gz
// │   └──[AppName].app
// │      └── Contents
// │          └── ...
// └── ...
#[cfg(feature = "updater")]
#[cfg(target_os = "macos")]
fn copy_files_and_run<R: Read + Seek>(archive_buffer: R, extract_path: &Path) -> Result {
	let mut extracted_files: Vec<PathBuf> = Vec::new();

	// extract the buffer to the tmp_dir
	// we extract our signed archive into our final directory without any temp file
	let mut extractor = Extract::from_cursor(archive_buffer, ArchiveFormat::Tar(Some(Compression::Gz)));
	// the first file in the tar.gz will always be
	// <app_name>/Contents
	let all_files = extractor.files()?;
	let tmp_dir = tempfile::Builder::new().prefix("millennium_current_app").tempdir()?;

	// create backup of our current app
	Move::from_source(extract_path).to_dest(tmp_dir.path())?;

	// extract all the files
	for file in all_files {
		// skip the first folder (should be the app name)
		let collected_path: PathBuf = file.iter().skip(1).collect();
		let extraction_path = extract_path.join(collected_path);

		// if something went wrong during the extraction, we should restore previous app
		if let Err(err) = extractor.extract_file(&extraction_path, &file) {
			for file in extracted_files {
				// delete all the files we extracted
				if file.is_dir() {
					std::fs::remove_dir(file)?;
				} else {
					std::fs::remove_file(file)?;
				}
			}
			Move::from_source(tmp_dir.path()).to_dest(extract_path)?;
			return Err(Error::Extract(err.to_string()));
		}

		extracted_files.push(extraction_path);
	}

	Ok(())
}

/// Returns a target os
/// We do not use a helper function like the target_triple
/// from millennium-utils because this function return `None` if
/// the updater do not support the platform.
///
/// Available target: `linux, darwin, win32, win64`
pub fn get_updater_target() -> Option<String> {
	if cfg!(target_os = "linux") {
		Some("linux".into())
	} else if cfg!(target_os = "macos") {
		Some("darwin".into())
	} else if cfg!(target_os = "windows") {
		if cfg!(target_pointer_width = "32") { Some("win32".into()) } else { Some("win64".into()) }
	} else {
		None
	}
}

/// Get the extract_path from the provided executable_path
#[allow(unused_variables)]
pub fn extract_path_from_executable(env: &Env, executable_path: &Path) -> PathBuf {
	// Return the path of the current executable by default
	// Example C:\Program Files\My App\
	let extract_path = executable_path.parent().map(PathBuf::from).expect("Can't determine extract path");

	// MacOS example binary is in /Applications/TestApp.app/Contents/MacOS/myApp
	// We need to get /Applications/<app>.app
	// todo(lemarier): Need a better way here
	// Maybe we could search for <*.app> to get the right path
	#[cfg(target_os = "macos")]
	if extract_path.display().to_string().contains("Contents/MacOS") {
		return extract_path
			.parent()
			.map(PathBuf::from)
			.expect("Unable to find the extract path")
			.parent()
			.map(PathBuf::from)
			.expect("Unable to find the extract path");
	}

	// We should use APPIMAGE exposed env variable
	// This is where our APPIMAGE should sit and should be replaced
	#[cfg(target_os = "linux")]
	if let Some(app_image_path) = &env.appimage {
		return PathBuf::from(app_image_path);
	}

	extract_path
}

// Convert base64 to string and prevent failing
fn base64_to_string(base64_string: &str) -> Result<String> {
	let decoded_string = &decode(base64_string)?;
	let result = from_utf8(decoded_string)?.to_string();
	Ok(result)
}

// Validate signature
// need to be public because its been used
// by our tests in the bundler
//
// NOTE: The buffer position is not reset.
pub fn verify_signature<R>(archive_reader: &mut R, release_signature: &str, pub_key: &str) -> Result<bool>
where
	R: Read
{
	// we need to convert the pub key
	let pub_key_decoded = base64_to_string(pub_key)?;
	let public_key = PublicKey::decode(&pub_key_decoded)?;
	let signature_base64_decoded = base64_to_string(release_signature)?;
	let signature = Signature::decode(&signature_base64_decoded)?;

	// read all bytes until EOF in the buffer
	let mut data = Vec::new();
	archive_reader.read_to_end(&mut data)?;

	// Validate signature or bail out
	public_key.verify(&data, &signature, true)?;
	Ok(true)
}
