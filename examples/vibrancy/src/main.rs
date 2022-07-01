#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use std::sync::Mutex;

use millennium::Manager;
use millennium_plugin_vibrancy::{apply_acrylic, apply_mica, clear_acrylic, clear_mica};

struct AcrylicState {
	mica: Mutex<bool>,
	acrylic: Mutex<bool>
}

#[millennium::command]
fn toggle_mica(window: millennium::Window, state: millennium::State<AcrylicState>) {
	let mut acrylic_state = state.acrylic.lock().unwrap();
	if *acrylic_state {
		clear_acrylic(&window);
		*acrylic_state = false;
	}

	let mut mica_state = state.mica.lock().unwrap();
	*mica_state = !*mica_state;
	if *mica_state {
		apply_mica(&window);
	} else {
		clear_mica(&window);
	}
}

#[millennium::command]
fn toggle_acrylic(window: millennium::Window, state: millennium::State<AcrylicState>) {
	let mut mica_state = state.mica.lock().unwrap();
	if *mica_state {
		clear_mica(&window);
		*mica_state = false;
	}

	let mut acrylic_state = state.acrylic.lock().unwrap();
	*acrylic_state = !*acrylic_state;
	if *acrylic_state {
		apply_acrylic(&window, None);
	} else {
		clear_acrylic(&window);
	}
}

fn main() {
	millennium::Builder::default()
		.manage(AcrylicState {
			mica: Mutex::new(true),
			acrylic: Mutex::new(false)
		})
		.invoke_handler(millennium::generate_handler![toggle_mica, toggle_acrylic])
		.setup(|app| {
			let main = app.get_window("main").unwrap();
			apply_mica(&main);

			Ok(())
		})
		.run(millennium::generate_context!())
		.expect("error while running application");
}
