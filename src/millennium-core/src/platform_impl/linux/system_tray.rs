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

use std::path::PathBuf;

use glib::Sender;
use gtk::{prelude::WidgetExt, AccelGroup};
use libappindicator::{AppIndicator, AppIndicatorStatus};

use super::{menu::Menu, window::WindowRequest, WindowId};
use crate::{
	error::OsError,
	event_loop::EventLoopWindowTarget,
	system_tray::{Icon, SystemTray as RootSystemTray}
};

pub struct SystemTrayBuilder {
	pub(crate) temp_icon_dir: Option<PathBuf>,
	tray_menu: Option<Menu>,
	icon: Icon
}

impl SystemTrayBuilder {
	#[inline]
	pub fn new(icon: Icon, tray_menu: Option<Menu>) -> Self {
		Self { temp_icon_dir: None, tray_menu, icon }
	}

	#[inline]
	pub fn build<T: 'static>(self, window_target: &EventLoopWindowTarget<T>) -> Result<RootSystemTray, OsError> {
		let mut app_indicator = AppIndicator::new("millennium-core", "");

		let (parent_path, icon_path) = temp_icon_path(self.temp_icon_dir.as_ref()).expect("failed to create temp folder for system tray icon");

		self.icon.inner.write_to_png(&icon_path);

		app_indicator.set_icon_theme_path(&parent_path.to_string_lossy());
		app_indicator.set_icon_full(&icon_path.to_string_lossy(), "icon");

		let sender = window_target.p.window_requests_tx.clone();

		if let Some(tray_menu) = self.tray_menu.clone() {
			let menu = &mut tray_menu.into_gtkmenu(&sender, &AccelGroup::new(), WindowId::dummy());

			app_indicator.set_menu(menu);
			menu.show_all();
		}

		app_indicator.set_status(AppIndicatorStatus::Active);

		Ok(RootSystemTray(SystemTray {
			temp_icon_dir: self.temp_icon_dir,
			app_indicator,
			sender,
			icon_path
		}))
	}
}

pub struct SystemTray {
	temp_icon_dir: Option<PathBuf>,
	app_indicator: AppIndicator,
	sender: Sender<(WindowId, WindowRequest)>,
	icon_path: PathBuf
}

impl SystemTray {
	pub fn set_icon(&mut self, icon: Icon) {
		let (parent_path, icon_path) = temp_icon_path(self.temp_icon_dir.as_ref()).expect("Failed to create a temp folder for icon");
		icon.inner.write_to_png(&icon_path);

		self.app_indicator.set_icon_theme_path(&parent_path.to_string_lossy());
		self.app_indicator.set_icon_full(&icon_path.to_string_lossy(), "icon");
		self.icon_path = icon_path;
	}

	pub fn set_menu(&mut self, tray_menu: &Menu) {
		let mut menu = tray_menu.clone().into_gtkmenu(&self.sender, &AccelGroup::new(), WindowId::dummy());

		self.app_indicator.set_menu(&mut menu);
		menu.show_all();
	}
}

impl Drop for SystemTray {
	fn drop(&mut self) {
		let _ = std::fs::remove_file(self.icon_path.clone());
	}
}

fn temp_icon_path(temp_icon_dir: Option<&PathBuf>) -> std::io::Result<(PathBuf, PathBuf)> {
	let parent_path = match temp_icon_dir.as_ref() {
		Some(path) => path.to_path_buf(),
		None => dirs_next::runtime_dir().unwrap_or_else(|| std::env::temp_dir()).join("millennium-core")
	};

	std::fs::create_dir_all(&parent_path)?;
	let icon_path = parent_path.join(uuid::Uuid::new_v4());
	Ok((parent_path, icon_path))
}

#[test]
fn temp_icon_path_preference_order() {
	let runtime_dir = option_env!("XDG_RUNTIME_DIR");
	let override_dir = PathBuf::from("/tmp/millennium-tests");

	let (dir1, _file1) = temp_icon_path(Some(&override_dir)).unwrap();
	let (dir2, _file2) = temp_icon_path(None).unwrap();
	std::env::remove_var("XDG_RUNTIME_DIR");
	let (dir3, _file3) = temp_icon_path(None).unwrap();

	assert_eq!(dir1, override_dir);
	if let Some(runtime_dir) = runtime_dir {
		std::env::set_var("XDG_RUNTIME_DIR", runtime_dir);
		assert_eq!(dir2, PathBuf::from(format!("{}/millennium", runtime_dir)));
	}

	assert_eq!(dir3, PathBuf::from("/tmp/millennium"));
}
