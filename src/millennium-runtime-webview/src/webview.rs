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

#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
mod imp {
	use std::rc::Rc;

	pub type Webview = Rc<webkit2gtk::WebView>;
}

#[cfg(target_os = "macos")]
mod imp {
	use cocoa::base::id;

	pub struct Webview {
		pub webview: id,
		pub manager: id,
		pub ns_window: id
	}
}

#[cfg(windows)]
mod imp {
	use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller;
	pub struct Webview {
		pub controller: ICoreWebView2Controller
	}
}

pub use imp::*;
