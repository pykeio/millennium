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
	collections::{HashMap, HashSet},
	fmt,
	path::{Path, PathBuf},
	sync::{Arc, Mutex}
};

pub use glob::Pattern;
use millennium_utils::{
	config::{Config, FsAllowlistScope},
	Env, PackageInfo
};
use uuid::Uuid;

use crate::api::path::parse as parse_path;

/// Scope change event.
#[derive(Debug, Clone)]
pub enum Event {
	/// A path has been allowed.
	PathAllowed(PathBuf),
	/// A path has been forbidden.
	PathForbidden(PathBuf)
}

type EventListener = Box<dyn Fn(&Event) + Send>;

/// Scope for filesystem access.
#[derive(Clone)]
pub struct Scope {
	allowed_patterns: Arc<Mutex<HashSet<Pattern>>>,
	forbidden_patterns: Arc<Mutex<HashSet<Pattern>>>,
	event_listeners: Arc<Mutex<HashMap<Uuid, EventListener>>>
}

impl fmt::Debug for Scope {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Scope")
			.field(
				"allowed_patterns",
				&self.allowed_patterns.lock().unwrap().iter().map(|p| p.as_str()).collect::<Vec<&str>>()
			)
			.field(
				"forbidden_patterns",
				&self.forbidden_patterns.lock().unwrap().iter().map(|p| p.as_str()).collect::<Vec<&str>>()
			)
			.finish()
	}
}

fn push_pattern<P: AsRef<Path>>(list: &mut HashSet<Pattern>, pattern: P) -> crate::Result<()> {
	let path: PathBuf = pattern.as_ref().components().collect();
	list.insert(Pattern::new(&path.to_string_lossy())?);
	#[cfg(windows)]
	{
		list.insert(Pattern::new(&format!("\\\\?\\{}", path.display()))?);
	}
	Ok(())
}

impl Scope {
	/// Creates a new scope from a `FsAllowlistScope` configuration.
	pub(crate) fn for_fs_api(config: &Config, package_info: &PackageInfo, env: &Env, scope: &FsAllowlistScope) -> crate::Result<Self> {
		let mut allowed_patterns = HashSet::new();
		for path in scope.allowed_paths() {
			if let Ok(path) = parse_path(config, package_info, env, path) {
				push_pattern(&mut allowed_patterns, path)?;
			}
		}

		let mut forbidden_patterns = HashSet::new();
		if let Some(forbidden_paths) = scope.forbidden_paths() {
			for path in forbidden_paths {
				if let Ok(path) = parse_path(config, package_info, env, path) {
					push_pattern(&mut forbidden_patterns, path)?;
				}
			}
		}

		Ok(Self {
			allowed_patterns: Arc::new(Mutex::new(allowed_patterns)),
			forbidden_patterns: Arc::new(Mutex::new(forbidden_patterns)),
			event_listeners: Default::default()
		})
	}

	/// The list of allowed patterns.
	pub fn allowed_patterns(&self) -> HashSet<Pattern> {
		self.allowed_patterns.lock().unwrap().clone()
	}

	/// The list of forbidden patterns.
	pub fn forbidden_patterns(&self) -> HashSet<Pattern> {
		self.forbidden_patterns.lock().unwrap().clone()
	}

	/// Listen to an event on this scope.
	pub fn listen<F: Fn(&Event) + Send + 'static>(&self, f: F) -> Uuid {
		let id = Uuid::new_v4();
		self.event_listeners.lock().unwrap().insert(id, Box::new(f));
		id
	}

	fn trigger(&self, event: Event) {
		for listener in self.event_listeners.lock().unwrap().values() {
			listener(&event);
		}
	}

	/// Extend the allowed patterns with the given directory.
	///
	/// After this function has been called, the frontend will be able to use
	/// the Millennium API to read the directory and all of its files and
	/// subdirectories.
	pub fn allow_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) -> crate::Result<()> {
		let path = path.as_ref().to_path_buf();
		{
			let mut list = self.allowed_patterns.lock().unwrap();

			// allow the directory to be read
			push_pattern(&mut list, &path)?;
			// allow its files and subdirectories to be read
			push_pattern(&mut list, path.join(if recursive { "**" } else { "*" }))?;
		}

		self.trigger(Event::PathAllowed(path));
		Ok(())
	}

	/// Extend the allowed patterns with the given file path.
	///
	/// After this function has been called, the frontend will be able to use
	/// the Millennium API to read the contents of this file.
	pub fn allow_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
		let path = path.as_ref();
		push_pattern(&mut self.allowed_patterns.lock().unwrap(), &path)?;
		self.trigger(Event::PathAllowed(path.to_path_buf()));
		Ok(())
	}

	/// Set the given directory path to be forbidden by this scope.
	///
	/// **Note**: this takes precedence over allowed paths, so its access gets
	/// denied **always**.
	pub fn forbid_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) -> crate::Result<()> {
		let path = path.as_ref().to_path_buf();
		{
			let mut list = self.forbidden_patterns.lock().unwrap();

			// forbid the directory to be read
			push_pattern(&mut list, &path)?;
			// forbid its files and subdirectories to be read
			push_pattern(&mut list, path.join(if recursive { "**" } else { "*" }))?;
		}

		self.trigger(Event::PathForbidden(path));
		Ok(())
	}

	/// Set the given file path to be forbidden by this scope.
	///
	/// **Note**: this takes precedence over allowed paths, so its access gets
	/// denied **always**.
	pub fn forbid_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
		let path = path.as_ref();
		push_pattern(&mut self.forbidden_patterns.lock().unwrap(), &path)?;
		self.trigger(Event::PathForbidden(path.to_path_buf()));
		Ok(())
	}

	/// Determines if the given path is allowed on this scope.
	pub fn is_allowed<P: AsRef<Path>>(&self, path: P) -> bool {
		let path = path.as_ref();
		let path = if !path.exists() {
			crate::Result::Ok(path.to_path_buf())
		} else {
			std::fs::canonicalize(path).map_err(Into::into)
		};

		if let Ok(path) = path {
			let path: PathBuf = path.components().collect();

			let forbidden = self.forbidden_patterns.lock().unwrap().iter().any(|p| p.matches_path(&path));
			if forbidden {
				false
			} else {
				let allowed = self.allowed_patterns.lock().unwrap().iter().any(|p| p.matches_path(&path));
				allowed
			}
		} else {
			false
		}
	}
}
