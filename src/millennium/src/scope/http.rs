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

use glob::Pattern;
use millennium_utils::config::HttpAllowlistScope;

/// Scope for filesystem access.
#[derive(Debug, Clone)]
pub struct Scope {
	allowed_urls: Vec<Pattern>
}

impl Scope {
	/// Creates a new scope from the allowlist's `http` scope configuration.
	#[allow(dead_code)]
	pub(crate) fn for_http_api(scope: &HttpAllowlistScope) -> Self {
		Self {
			allowed_urls: scope
				.0
				.iter()
				.map(|url| glob::Pattern::new(url.as_str()).unwrap_or_else(|_| panic!("scoped URL is not a valid glob pattern: `{}`", url)))
				.collect()
		}
	}

	/// Determines if the given URL is allowed on this scope.
	pub fn is_allowed(&self, url: &url::Url) -> bool {
		self.allowed_urls.iter().any(|allowed| allowed.matches(url.as_str()))
	}
}

#[cfg(test)]
mod tests {
	use millennium_utils::config::HttpAllowlistScope;

	#[test]
	fn is_allowed() {
		// plain URL
		let scope = super::Scope::for_http_api(&HttpAllowlistScope(vec!["http://localhost:8080".parse().unwrap()]));
		assert!(scope.is_allowed(&"http://localhost:8080".parse().unwrap()));
		assert!(scope.is_allowed(&"http://localhost:8080/".parse().unwrap()));

		assert!(!scope.is_allowed(&"http://localhost:8080/file".parse().unwrap()));
		assert!(!scope.is_allowed(&"http://localhost:8080/path/to/asset.png".parse().unwrap()));
		assert!(!scope.is_allowed(&"https://localhost:8080".parse().unwrap()));
		assert!(!scope.is_allowed(&"http://localhost:8081".parse().unwrap()));
		assert!(!scope.is_allowed(&"http://local:8080".parse().unwrap()));

		// URL with fixed path
		let scope = super::Scope::for_http_api(&HttpAllowlistScope(vec!["http://localhost:8080/file.png".parse().unwrap()]));

		assert!(scope.is_allowed(&"http://localhost:8080/file.png".parse().unwrap()));

		assert!(!scope.is_allowed(&"http://localhost:8080".parse().unwrap()));
		assert!(!scope.is_allowed(&"http://localhost:8080/file".parse().unwrap()));
		assert!(!scope.is_allowed(&"http://localhost:8080/file.png/other.jpg".parse().unwrap()));

		// URL with glob pattern
		let scope = super::Scope::for_http_api(&HttpAllowlistScope(vec!["http://localhost:8080/*.png".parse().unwrap()]));

		assert!(scope.is_allowed(&"http://localhost:8080/file.png".parse().unwrap()));
		assert!(scope.is_allowed(&"http://localhost:8080/assets/file.png".parse().unwrap()));

		assert!(!scope.is_allowed(&"http://localhost:8080/file.jpeg".parse().unwrap()));

		let scope = super::Scope::for_http_api(&HttpAllowlistScope(vec!["http://*".parse().unwrap()]));

		assert!(scope.is_allowed(&"http://something.else".parse().unwrap()));
		assert!(!scope.is_allowed(&"http://something.else/path/to/file".parse().unwrap()));
		assert!(!scope.is_allowed(&"https://something.else".parse().unwrap()));

		let scope = super::Scope::for_http_api(&HttpAllowlistScope(vec!["http://**".parse().unwrap()]));

		assert!(scope.is_allowed(&"http://something.else".parse().unwrap()));
		assert!(scope.is_allowed(&"http://something.else/path/to/file".parse().unwrap()));
	}
}
