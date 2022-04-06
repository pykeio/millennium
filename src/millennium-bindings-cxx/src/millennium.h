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

#ifdef __cplusplus
extern "C" {
#endif

typedef void *MillenniumBuilder;
typedef void *MillenniumWindowBuilder;

typedef struct MillenniumInvoke {
	void *message;
	void *resolver;
} MillenniumInvoke;

extern MillenniumBuilder millennium_builder_new(void);

extern void millennium_builder_run(MillenniumBuilder builder);

extern void millennium_builder_setup(MillenniumBuilder builder, void (*setup)(void *opaque, void *app), void *opaque);

extern void millennium_builder_invoke_handler(MillenniumBuilder builder, void (*handler)(void *opaque, MillenniumInvoke *invoke), void *opaque);

extern const char *millennium_invoke_message_command(void *message);

extern MillenniumWindowBuilder millennium_window_builder_new(void *app, const char *title, const char *url);

extern void millennium_window_builder_title(MillenniumWindowBuilder builder, const char *title);

extern void millennium_window_builder_center(MillenniumWindowBuilder builder);

extern void millennium_window_builder_build(MillenniumWindowBuilder builder);

#ifdef __cplusplus
}
#endif

#endif
