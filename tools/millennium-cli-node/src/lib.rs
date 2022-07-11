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

use napi::{
	threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
	Error, JsFunction, Result, Status
};

#[napi_derive::napi]
pub fn run(args: Vec<String>, bin_name: Option<String>, callback: JsFunction) -> Result<()> {
	let function: ThreadsafeFunction<bool, ErrorStrategy::CalleeHandled> =
		callback.create_threadsafe_function(0, |ctx| ctx.env.get_boolean(ctx.value).map(|v| vec![v]))?;

	std::thread::spawn(move || match millennium_cli::try_run(args, bin_name) {
		Ok(_) => function.call(Ok(true), ThreadsafeFunctionCallMode::Blocking),
		Err(e) => function.call(Err(Error::new(Status::GenericFailure, format!("{:#}", e))), ThreadsafeFunctionCallMode::Blocking)
	});

	Ok(())
}

#[napi_derive::napi]
pub fn log_error(error: String) {
	log::error!("{}", error);
}
