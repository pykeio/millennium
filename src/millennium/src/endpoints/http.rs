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

#![allow(unused_imports)]

#[cfg(http_request)]
use std::{
	collections::HashMap,
	sync::{Arc, Mutex}
};

use millennium_macros::{command_enum, module_command_handler, CommandModule};
use serde::Deserialize;

use super::InvokeContext;
#[cfg(http_request)]
use crate::api::http::{ClientBuilder, HttpRequestBuilder, ResponseData};
use crate::Runtime;
#[cfg(not(http_request))]
type ClientBuilder = ();
#[cfg(not(http_request))]
type HttpRequestBuilder = ();
#[cfg(not(http_request))]
#[allow(dead_code)]
type ResponseData = ();

type ClientId = u32;
#[cfg(http_request)]
type ClientStore = Arc<Mutex<HashMap<ClientId, crate::api::http::Client>>>;

#[cfg(http_request)]
fn clients() -> &'static ClientStore {
	use once_cell::sync::Lazy;
	static STORE: Lazy<ClientStore> = Lazy::new(Default::default);
	&STORE
}

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[cmd(async)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
	/// Create a new HTTP client.
	#[cmd(http_request, "http > request")]
	CreateClient { options: Option<ClientBuilder> },
	/// Drop a HTTP client.
	#[cmd(http_request, "http > request")]
	DropClient { client: ClientId },
	/// The HTTP request API.
	#[cmd(http_request, "http > request")]
	HttpRequest { client: ClientId, options: Box<HttpRequestBuilder> }
}

impl Cmd {
	#[module_command_handler(http_request)]
	async fn create_client<R: Runtime>(_context: InvokeContext<R>, options: Option<ClientBuilder>) -> super::Result<ClientId> {
		let client = options.unwrap_or_default().build()?;
		let mut store = clients().lock().unwrap();
		let id = rand::random::<ClientId>();
		store.insert(id, client);
		Ok(id)
	}

	#[module_command_handler(http_request)]
	async fn drop_client<R: Runtime>(_context: InvokeContext<R>, client: ClientId) -> super::Result<()> {
		let mut store = clients().lock().unwrap();
		store.remove(&client);
		Ok(())
	}

	#[module_command_handler(http_request)]
	async fn http_request<R: Runtime>(context: InvokeContext<R>, client_id: ClientId, options: Box<HttpRequestBuilder>) -> super::Result<ResponseData> {
		use crate::Manager;
		let scopes = context.window.state::<crate::Scopes>();
		if scopes.http.is_allowed(&options.url) {
			let client = clients()
				.lock()
				.unwrap()
				.get(&client_id)
				.ok_or_else(|| crate::Error::HttpClientNotInitialized.into_anyhow())?
				.clone();
			let options = *options;
			if let Some(crate::api::http::Body::Form(form)) = &options.body {
				for value in form.0.values() {
					if let crate::api::http::FormPart::File {
						file: crate::api::http::FilePart::Path(path),
						..
					} = value
					{
						if crate::api::file::SafePathBuf::new(path.clone()).is_err() || !scopes.fs.is_allowed(&path) {
							return Err(crate::Error::PathNotAllowed(path.clone()).into_anyhow());
						}
					}
				}
			}
			let response = client.send(options).await?;
			Ok(response.read().await?)
		} else {
			Err(crate::Error::UrlNotAllowed(options.url).into_anyhow())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{ClientBuilder, ClientId};

	#[millennium_macros::module_command_test(http_request, "http > request")]
	#[quickcheck_macros::quickcheck]
	fn create_client(options: Option<ClientBuilder>) {
		assert!(crate::async_runtime::block_on(super::Cmd::create_client(crate::test::mock_invoke_context(), options)).is_ok());
	}

	#[millennium_macros::module_command_test(http_request, "http > request")]
	#[quickcheck_macros::quickcheck]
	fn drop_client(client_id: ClientId) {
		crate::async_runtime::block_on(async move {
			assert!(super::Cmd::drop_client(crate::test::mock_invoke_context(), client_id).await.is_ok());
			let id = super::Cmd::create_client(crate::test::mock_invoke_context(), None).await.unwrap();
			assert!(super::Cmd::drop_client(crate::test::mock_invoke_context(), id).await.is_ok());
		});
	}
}
