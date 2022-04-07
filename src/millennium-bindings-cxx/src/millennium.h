/**
 * Copyright 2022 pyke.io
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#ifndef _MILLENNIUM_H__
#define _MILLENNIUM_H__

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void *MillenniumBuilder;
typedef void *MillenniumWindowBuilder;

typedef struct MillenniumInvoke {
	void *message;
	void *resolver;
} MillenniumInvoke;

extern const char *millennium_last_error(void);

extern MillenniumBuilder millennium_builder_new(void);

extern int millennium_builder_run(MillenniumBuilder builder);

extern int millennium_builder_setup(MillenniumBuilder builder, void (*setup)(void *opaque, void *app), void *opaque);

extern int millennium_builder_invoke_handler(MillenniumBuilder builder, void (*handler)(void *opaque, MillenniumInvoke *invoke), void *opaque);

extern int millennium_builder_free(MillenniumBuilder builder);

extern const char *millennium_invoke_message_command(void *message);

extern MillenniumWindowBuilder millennium_window_builder_new(void *app, const char *title, const char *url, uint8_t is_external);

extern int millennium_window_builder_title(MillenniumWindowBuilder builder, const char *title);

extern int millennium_window_builder_center(MillenniumWindowBuilder builder);

extern void *millennium_window_builder_build(MillenniumWindowBuilder builder);

#ifdef __cplusplus
}

#include <stdexcept>

#ifndef MILLENNIUM_NAMESPACE
	#define MILLENNIUM_NAMESPACE millennium
#endif

#define millenniumHandleException(v) \
	if (v != 0) { \
		const char *error = millennium_last_error(); \
		if (error != NULL) { \
			throw std::runtime_error(error); \
		} else { \
			throw std::runtime_error("Unknown error"); \
		} \
	}

namespace MILLENNIUM_NAMESPACE {

class Builder {
	public:
		Builder() {
			MillenniumBuilder builder = millennium_builder_new();
			if (builder == NULL)
				throw std::runtime_error(millennium_last_error());
			this->builder = builder;
		}

		~Builder() {
			millennium_builder_free(builder);
		}

		template<typename F>
		inline Builder &setup(F &&callback) {
			millenniumHandleException(millennium_builder_setup(builder, [](void *opaque, void *app) {
				((F &&)opaque)(app);
			}, (void *)callback))
			return *this;
		}

		template<typename F>
		inline Builder &invoke_handler(F &&callback) {
			millenniumHandleException(millennium_builder_invoke_handler(builder, [](void *opaque, MillenniumInvoke *invoke) {
				((F &&)opaque)(invoke);
			}, (void *)callback))
			return *this;
		}

		inline Builder &run() {
			millenniumHandleException(millennium_builder_run(builder));
			return *this;
		}
	private:
		MillenniumBuilder builder;
};

class WindowBuilder {
	public:
		WindowBuilder(void *app, const char *title, const char *url, bool is_external = false) {
			MillenniumWindowBuilder builder = millennium_window_builder_new(app, title, url, (uint8_t)is_external);
			if (builder == NULL)
				throw std::runtime_error(millennium_last_error());
			this->builder = builder;
		}

		inline WindowBuilder &title(const char *title) {
			millenniumHandleException(millennium_window_builder_title(builder, title));
			return *this;
		}

		inline WindowBuilder &center() {
			millenniumHandleException(millennium_window_builder_center(builder));
			return *this;
		}

		void *build() {
			void *window = millennium_window_builder_build(builder);
			if (window == NULL)
				throw std::runtime_error(millennium_last_error());
			return window;
		}
	private:
		MillenniumWindowBuilder builder;
};

}
#endif

#endif
