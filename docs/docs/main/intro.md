---
sidebar_position: 1
---

# Introduction to Millennium
Millennium is an open-source cross-platform webview framework written in Rust. It is a fork of [Tauri](https://tauri.studio/) and related modules with some added features and changes.

:::note

Help the future of Tauri by donating to Tauri [here](https://opencollective.com/tauri). Millennium would not be possible without the awesome work from the Tauri team 💖

:::

Millennium leverages the system's pre-included webview libraries (WebView2 on Windows, WebKit on macOS, and WebKitGTK on Linux) for ultra-small binaries; why ship an entire web browser with your application when there's a perfectly usable one already installed on the system?

## Philosophy

### Security First
In today's world, every honest threat model assumes that the user's device has already been compromised. This puts app developers in a complicated situation because if the device is already at risk, how can the software be trusted?

Defense in depth is the approach we've taken. We want you to be able to take every precaution possible to minimize the surface area you present to attackers. Millennium lets you choose which API endpoints to ship, whether or not you want a localhost server built into your app, and it even randomizes functional handles at runtime. These and other techniques form a secure baseline that empowers you and your users.

Slowing down attackers by making static attacks crushingly difficult and isolating systems from one another is the name of the game. And if you are coming from the Electron ecosystem — rest assured — by default Millennium only ships a single bundled binary, no ASAR files.

By choosing to build Millennium with security as a guiding force, we give you every opportunity to take a proactive security posture.

### Polyglots, not Silos
Most contemporary frameworks use a single language paradigm and are therefore trapped in a bubble of knowledge and idiom. This can work well for certain niche applications, but it also fosters a kind of tribalism.

This can be seen in the way that the React, Angular, and Vue development communities huddle on their stacks, ultimately breeding very little cross-pollination.

This same situation can be seen in the Rust vs. Node vs. C++ battlefields, where hardliners take their stances and refuse to collaborate across communities.

Today, Millennium uses Rust for the backend with C++ bindings in the works, but in the not too distant future, other backends like Go, Nim, Python, C#, etc. will be possible. Since our API can be implemented in any language with C interop, full compliance is only a PR away.

### Honest Open Source
None of this would make any sense without a community. Today, software communities are amazing places where people help each other and make awesome things - open source is a very big part of that.

Open source means different things to different people, but most will agree that it serves to support freedom. When software doesn't respect your rights, then it can seem unfair and potentially compromise your freedoms by operating in unethical ways.

This is why we are proud that FLOSS advocates can build applications with Millennium that are "certifiably" open source and can be included in FSF endorsed GNU/Linux distributions.

## Architecture
Millennium is composed of multiple packages working together to create a finished webview. There are 3 main components of Millennium: `millennium`, `millennium-runtime`, and `millennium-build`.

### The `millennium` package
The `millennium` package is the glue that binds everything together; it is responsible for all frontend APIs, security functions, asset loading, updating, and most importantly, interfacing with the runtime via a high-level API.

### The `millennium-runtime` package
Millennium Runtimes do most of the low-level work. They provide the low-level APIs to control the window and webview that is then used by Millennium. Runtimes are one of Millennium's most unique traits: you can implement your own runtime and connect it to any Rust-interoperable webview engine you please.

By default, Millennium uses `millennium-webview` as its runtime, which uses `millennium-core` for windowing. `millennium-webview` is configured as a runtime via the `millennium-runtime-webview` crate.

### The `millennium-build` package
`millennium-build` wraps your app up into a neat little package. It performs compile-time checks to make sure the right Millennium features are enabled and bundles all assets into one single executable.

This executable is not ready for deployment just yet — that is handled by `millennium-bundler`, which creates an OS package (e.g. `.dmg`, `.msi`, `.AppImage`) that will install your application in a proper place and install the required webview backends if not installed.
