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

// custom Millennium Webview types
mod mime_type;
mod request;
mod response;

// re-expose default http types
pub use http::{header, method, status, uri::InvalidUri, version, Uri};
// re-export httprange helper as it can be useful and we need it locally
pub use http_range::HttpRange;

pub use self::{
	mime_type::MimeType,
	request::{Request, RequestParts},
	response::{Builder as ResponseBuilder, Response, ResponseParts}
};
