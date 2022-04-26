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

use std::{collections::HashSet, ffi::c_void, ptr::null_mut, rc::Rc, sync::RwLock};

use jni::{
	objects::{JClass, JObject},
	sys::jobject,
	JNIEnv
};
use once_Cell::sync::Lazy;

use super::{WebContext, WebViewAttributes};
use crate::{application::window::Window, Result};
static IPC: Lazy<RwLock<UnsafeIpc>> = Lazy::new(|| RwLock::new(UnsafeIpc(null_mut())));

pub struct InnerWebView {
	pub window: Rc<Window>,
	pub attributes: WebViewAttributes
}

impl InnerWebView {
	pub fn new(window: Rc<Window>, attributes: WebViewAttributes, _web_context: Option<&mut WebContext>) -> Result<Self> {
		Ok(Self { window, attributes })
	}

	pub fn print(&self) {}

	pub fn eval(&self, _js: &str) -> Result<()> {
		Ok(())
	}

	pub fn focus(&self) {}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	pub fn open_devtools(&self) {}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	pub fn close_devtools(&self) {}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	pub fn is_devtools_open(&self) -> bool {
		false
	}

	pub fn run(self, env: JNIEnv, _jclass: JClass, jobject: JObject) -> Result<jobject> {
		let string_class = env.find_class("java/lang/String")?;
		let WebViewAttributes {
			url,
			custom_protocols,
			initialization_scripts,
			ipc_handler,
			..
		} = self.attributes;

		if let Some(i) = ipc_handler {
			let i = UnsafeIpc(Box::into_raw(Box::new(i)) as *mut _);
			let mut ipc = IPC.write().unwrap();
			*ipc = i;
		}

		if let Some(u) = url {
			let mut url_string = String::from(u.as_str());
			let schemes = custom_protocols.into_iter().map(|(s, _)| s).collect::<HashSet<_>>();
			let name = u.scheme();
			if schemes.contains(name) {
				url_string = u.as_str().replace(&format!("{}://", name), "https://millennium.pyke/");
			}
			let url = env.new_string(url_string)?;
			env.call_method(jobject, "loadUrl", "(Ljava/lang/String;)V", &[url.into()])?;
		}

		// Return initialization scripts
		let len = initialization_scripts.len();
		let scripts = env.new_object_array(len as i32, string_class, env.new_string("")?)?;
		for (idx, s) in initialization_scripts.into_iter().enumerate() {
			env.set_object_array_element(scripts, idx as i32, env.new_string(s)?)?;
		}
		Ok(scripts)
	}

	pub fn ipc_handler(window: &Window, arg: String) {
		let function = IPC.read().unwrap();
		unsafe {
			let ipc = function.0;
			if !ipc.is_null() {
				let ipc = &*(ipc as *mut Box<dyn Fn(&Window, String)>);
				ipc(window, arg)
			}
		}
	}
}

pub struct UnsafeIpc(*mut c_void);
unsafe impl Send for UnsafeIpc {}
unsafe impl Sync for UnsafeIpc {}

pub fn platform_webview_version() -> Result<String> {
	todo!()
}
