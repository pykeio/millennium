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

use std::{
	collections::HashMap,
	sync::{Arc, Mutex}
};

use millennium_runtime::{menu::MenuHash, UserEvent};
pub use millennium_runtime::{
	menu::{Menu, MenuEntry, MenuItem, MenuUpdate, Submenu, SystemTrayMenu, SystemTrayMenuEntry, SystemTrayMenuItem, TrayHandle},
	Icon, SystemTrayEvent
};
#[cfg(target_os = "macos")]
pub use millennium_webview::application::platform::macos::CustomMenuItemExtMacOS;
pub use millennium_webview::application::{
	event::TrayEvent,
	event_loop::EventLoopProxy,
	menu::{ContextMenu as MillenniumContextMenu, CustomMenuItem as MillenniumCustomMenuItem, MenuItem as MillenniumMenuItem},
	system_tray::Icon as MillenniumTrayIcon
};
use uuid::Uuid;

use crate::{Error, Message, Result, TrayMessage};

pub type SystemTrayEventHandler = Box<dyn Fn(&SystemTrayEvent) + Send>;
pub type SystemTrayEventListeners = Arc<Mutex<HashMap<Uuid, Arc<SystemTrayEventHandler>>>>;
pub type SystemTrayItems = Arc<Mutex<HashMap<u16, MillenniumCustomMenuItem>>>;

/// Wrapper around a [`millennium_webview::application::system_tray::Icon`] that can be created from a [`WindowIcon`].
pub struct TrayIcon(pub(crate) MillenniumTrayIcon);

impl TryFrom<Icon> for TrayIcon {
	type Error = Error;

	fn try_from(icon: Icon) -> std::result::Result<Self, Self::Error> {
		MillenniumTrayIcon::from_rgba(icon.rgba, icon.width, icon.height)
			.map(Self)
			.map_err(crate::icon_err)
	}
}

#[derive(Debug, Clone)]
pub struct SystemTrayHandle<T: UserEvent> {
	pub(crate) proxy: EventLoopProxy<super::Message<T>>
}

impl<T: UserEvent> TrayHandle for SystemTrayHandle<T> {
	fn set_icon(&self, icon: Icon) -> Result<()> {
		self.proxy
			.send_event(Message::Tray(TrayMessage::UpdateIcon(icon)))
			.map_err(|_| Error::FailedToSendMessage)
	}
	fn set_menu(&self, menu: SystemTrayMenu) -> Result<()> {
		self.proxy
			.send_event(Message::Tray(TrayMessage::UpdateMenu(menu)))
			.map_err(|_| Error::FailedToSendMessage)
	}
	fn update_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
		self.proxy
			.send_event(Message::Tray(TrayMessage::UpdateItem(id, update)))
			.map_err(|_| Error::FailedToSendMessage)
	}
	#[cfg(target_os = "macos")]
	fn set_icon_as_template(&self, is_template: bool) -> millennium_runtime::Result<()> {
		self.proxy
			.send_event(Message::Tray(TrayMessage::UpdateIconAsTemplate(is_template)))
			.map_err(|_| Error::FailedToSendMessage)
	}
}

impl From<SystemTrayMenuItem> for crate::MenuItemWrapper {
	fn from(item: SystemTrayMenuItem) -> Self {
		match item {
			SystemTrayMenuItem::Separator => Self(MillenniumMenuItem::Separator),
			_ => unimplemented!()
		}
	}
}

pub fn to_millennium_context_menu(custom_menu_items: &mut HashMap<MenuHash, MillenniumCustomMenuItem>, menu: SystemTrayMenu) -> MillenniumContextMenu {
	let mut tray_menu = MillenniumContextMenu::new();
	for item in menu.items {
		match item {
			SystemTrayMenuEntry::CustomItem(c) => {
				#[allow(unused_mut)]
				let mut item = tray_menu.add_item(crate::MenuItemAttributesWrapper::from(&c).0);
				#[cfg(target_os = "macos")]
				if let Some(native_image) = c.native_image {
					item.set_native_image(crate::NativeImageWrapper::from(native_image).0);
				}
				custom_menu_items.insert(c.id, item);
			}
			SystemTrayMenuEntry::NativeItem(i) => {
				tray_menu.add_native_item(crate::MenuItemWrapper::from(i).0);
			}
			SystemTrayMenuEntry::Submenu(submenu) => {
				tray_menu.add_submenu(&submenu.title, submenu.enabled, to_millennium_context_menu(custom_menu_items, submenu.inner));
			}
		}
	}
	tray_menu
}
