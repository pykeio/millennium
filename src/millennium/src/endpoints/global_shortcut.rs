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

use millennium_macros::{command_enum, module_command_handler, CommandModule};
use serde::Deserialize;

use super::InvokeContext;
#[cfg(global_shortcut_all)]
use crate::runtime::GlobalShortcutManager;
use crate::{api::ipc::CallbackFn, Runtime};

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
	/// Register a global shortcut.
	#[cmd(global_shortcut_all, "globalShortcut > all")]
	Register { shortcut: String, handler: CallbackFn },
	/// Register a list of global shortcuts.
	#[cmd(global_shortcut_all, "globalShortcut > all")]
	RegisterAll { shortcuts: Vec<String>, handler: CallbackFn },
	/// Unregister a global shortcut.
	#[cmd(global_shortcut_all, "globalShortcut > all")]
	Unregister { shortcut: String },
	/// Unregisters all registered shortcuts.
	#[cmd(global_shortcut_all, "globalShortcut > all")]
	UnregisterAll,
	/// Determines whether the given hotkey is registered or not.
	#[cmd(global_shortcut_all, "globalShortcut > all")]
	IsRegistered { shortcut: String }
}

impl Cmd {
	#[module_command_handler(global_shortcut_all)]
	fn register<R: Runtime>(context: InvokeContext<R>, shortcut: String, handler: CallbackFn) -> super::Result<()> {
		let mut manager = context.window.app_handle.global_shortcut_manager();
		register_shortcut(context.window, &mut manager, shortcut, handler)?;
		Ok(())
	}

	#[module_command_handler(global_shortcut_all)]
	fn register_all<R: Runtime>(context: InvokeContext<R>, shortcuts: Vec<String>, handler: CallbackFn) -> super::Result<()> {
		let mut manager = context.window.app_handle.global_shortcut_manager();
		for shortcut in shortcuts {
			register_shortcut(context.window.clone(), &mut manager, shortcut, handler)?;
		}
		Ok(())
	}

	#[module_command_handler(global_shortcut_all)]
	fn unregister<R: Runtime>(context: InvokeContext<R>, shortcut: String) -> super::Result<()> {
		context
			.window
			.app_handle
			.global_shortcut_manager()
			.unregister(&shortcut)
			.map_err(crate::error::into_anyhow)?;
		Ok(())
	}

	#[module_command_handler(global_shortcut_all)]
	fn unregister_all<R: Runtime>(context: InvokeContext<R>) -> super::Result<()> {
		context
			.window
			.app_handle
			.global_shortcut_manager()
			.unregister_all()
			.map_err(crate::error::into_anyhow)?;
		Ok(())
	}

	#[cfg(not(global_shortcut_all))]
	fn unregister_all<R: Runtime>(_: InvokeContext<R>) -> super::Result<()> {
		Err(crate::Error::ApiNotAllowlisted("globalShortcut > all".into()).into_anyhow())
	}

	#[module_command_handler(global_shortcut_all)]
	fn is_registered<R: Runtime>(context: InvokeContext<R>, shortcut: String) -> super::Result<bool> {
		context
			.window
			.app_handle
			.global_shortcut_manager()
			.is_registered(&shortcut)
			.map_err(crate::error::into_anyhow)
	}
}

#[cfg(global_shortcut_all)]
fn register_shortcut<R: Runtime>(window: crate::Window<R>, manager: &mut R::GlobalShortcutManager, shortcut: String, handler: CallbackFn) -> super::Result<()> {
	let accelerator = shortcut.clone();
	manager
		.register(&shortcut, move || {
			let callback_string = crate::api::ipc::format_callback(handler, &accelerator).expect("unable to serialize shortcut string to json");
			let _ = window.eval(callback_string.as_str());
		})
		.map_err(crate::error::into_anyhow)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use crate::api::ipc::CallbackFn;

	#[millennium_macros::module_command_test(global_shortcut_all, "globalShortcut > all")]
	#[quickcheck_macros::quickcheck]
	fn register(shortcut: String, handler: CallbackFn) {
		let ctx = crate::test::mock_invoke_context();
		super::Cmd::register(ctx.clone(), shortcut.clone(), handler).unwrap();
		assert!(super::Cmd::is_registered(ctx, shortcut).unwrap());
	}

	#[millennium_macros::module_command_test(global_shortcut_all, "globalShortcut > all")]
	#[quickcheck_macros::quickcheck]
	fn register_all(shortcuts: Vec<String>, handler: CallbackFn) {
		let ctx = crate::test::mock_invoke_context();
		super::Cmd::register_all(ctx.clone(), shortcuts.clone(), handler).unwrap();
		for shortcut in shortcuts {
			assert!(super::Cmd::is_registered(ctx.clone(), shortcut).unwrap(),);
		}
	}

	#[millennium_macros::module_command_test(global_shortcut_all, "globalShortcut > all")]
	#[quickcheck_macros::quickcheck]
	fn unregister(shortcut: String) {
		let ctx = crate::test::mock_invoke_context();
		super::Cmd::register(ctx.clone(), shortcut.clone(), CallbackFn(0)).unwrap();
		super::Cmd::unregister(ctx.clone(), shortcut.clone()).unwrap();
		assert!(!super::Cmd::is_registered(ctx, shortcut).unwrap());
	}

	#[millennium_macros::module_command_test(global_shortcut_all, "globalShortcut > all", runtime)]
	#[quickcheck_macros::quickcheck]
	fn unregister_all() {
		let shortcuts = vec!["CTRL+X".to_string(), "SUPER+C".to_string(), "D".to_string()];
		let ctx = crate::test::mock_invoke_context();
		super::Cmd::register_all(ctx.clone(), shortcuts.clone(), CallbackFn(0)).unwrap();
		super::Cmd::unregister_all(ctx.clone()).unwrap();
		for shortcut in shortcuts {
			assert!(!super::Cmd::is_registered(ctx.clone(), shortcut).unwrap(),);
		}
	}

	#[millennium_macros::module_command_test(global_shortcut_all, "globalShortcut > all")]
	#[quickcheck_macros::quickcheck]
	fn is_registered(shortcut: String) {
		let ctx = crate::test::mock_invoke_context();
		assert!(!super::Cmd::is_registered(ctx.clone(), shortcut.clone()).unwrap(),);
		super::Cmd::register(ctx.clone(), shortcut.clone(), CallbackFn(0)).unwrap();
		assert!(super::Cmd::is_registered(ctx, shortcut).unwrap());
	}
}
