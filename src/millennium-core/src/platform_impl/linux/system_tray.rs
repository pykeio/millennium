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
	tray_menu: Option<Menu>,
	app_indicator: AppIndicator,
	icon_path: PathBuf
}

impl SystemTrayBuilder {
	#[inline]
	pub fn new(icon: Icon, tray_menu: Option<Menu>) -> Self {
		let (parent_path, icon_path) = temp_icon_path().expect("Failed to create a temp folder for icon");
		icon.inner.write_to_png(&icon_path);

		let mut app_indicator = AppIndicator::new("MillenniumCoreApplication", "");
		app_indicator.set_icon_theme_path(&parent_path.to_string_lossy());
		app_indicator.set_icon_full(&icon_path.to_string_lossy(), "icon");

		Self { tray_menu, app_indicator, icon_path }
	}

	#[inline]
	pub fn build<T: 'static>(mut self, window_target: &EventLoopWindowTarget<T>) -> Result<RootSystemTray, OsError> {
		let sender = window_target.p.window_requests_tx.clone();

		if let Some(tray_menu) = self.tray_menu.clone() {
			let menu = &mut tray_menu.into_gtkmenu(&sender, &AccelGroup::new(), WindowId::dummy());

			self.app_indicator.set_menu(menu);
			menu.show_all();
		}

		self.app_indicator.set_status(AppIndicatorStatus::Active);

		Ok(RootSystemTray(SystemTray {
			app_indicator: self.app_indicator,
			sender,
			icon_path: self.icon_path
		}))
	}
}

pub struct SystemTray {
	app_indicator: AppIndicator,
	sender: Sender<(WindowId, WindowRequest)>,
	icon_path: PathBuf
}

impl SystemTray {
	pub fn set_icon(&mut self, icon: Icon) {
		let (parent_path, icon_path) = temp_icon_path().expect("Failed to create a temp folder for icon");
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

fn temp_icon_path() -> std::io::Result<(PathBuf, PathBuf)> {
	let mut parent_path = std::env::temp_dir();
	parent_path.push("millennium");
	std::fs::create_dir_all(&parent_path)?;
	let mut icon_path = parent_path.clone();
	icon_path.push(format!("{}.png", uuid::Uuid::new_v4()));
	Ok((parent_path, icon_path))
}
