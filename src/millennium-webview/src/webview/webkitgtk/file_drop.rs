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

use std::{cell::Cell, path::PathBuf, rc::Rc};

use gtk::prelude::*;
use webkit2gtk::WebView;

use crate::{application::window::Window, webview::FileDropEvent};

pub(crate) fn connect_drag_event(webview: Rc<WebView>, window: Rc<Window>, handler: Box<dyn Fn(&Window, FileDropEvent) -> bool>) {
	let listener = Rc::new((handler, Cell::new(None)));

	let listener_ref = listener.clone();
	let w = window.clone();
	webview.connect_drag_data_received(move |_, _, _, _, data, info, _| {
		if info == 2 {
			let uris = data
				.uris()
				.iter()
				.map(|gstr| {
					let path = gstr.as_str();
					PathBuf::from(path.to_string().strip_prefix("file://").unwrap_or(path))
				})
				.collect::<Vec<PathBuf>>();

			listener_ref.1.set(Some(uris.clone()));
			listener_ref.0(&w, FileDropEvent::Hovered(uris));
		} else {
			// drag_data_received is called twice, so we can ignore this signal
		}
	});

	let listener_ref = listener.clone();
	let w = window.clone();
	webview.connect_drag_drop(move |_, _, _, _, _| {
		let uris = listener_ref.1.take();
		if let Some(uris) = uris { listener_ref.0(&w, FileDropEvent::Dropped(uris)) } else { false }
	});

	let listener_ref = listener.clone();
	let w = window.clone();
	webview.connect_drag_leave(move |_, _, time| {
		if time == 0 {
			// The user cancelled the drag n drop
			listener_ref.0(&w, FileDropEvent::Cancelled);
		} else {
			// The user dropped the file on the window, but this will be handled
			// in connect_drag_drop instead
		}
	});

	// Called when a drag "fails" - we'll just emit a Cancelled event.
	let listener_ref = listener.clone();
	let w = window;
	webview.connect_drag_failed(move |_, _, _| gtk::Inhibit(listener_ref.0(&w, FileDropEvent::Cancelled)));
}
