#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

fn main() {
	millennium::Builder::default()
		.run(millennium::generate_context!())
		.expect("error while running application");
}
