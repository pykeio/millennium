---
sidebar_position: 1
---

# Introduction to Millennium
Millennium is an open-source cross-platform webview framework written in Rust. It is a fork of [Tauri](https://tauri.studio/) and related modules with added features and changes.

:::note

Help the future of Tauri by donating to Tauri [here](https://opencollective.com/tauri). Millennium would not be possible without the amazing work from the Tauri team 💖

:::

Millennium leverages the system's pre-included webview libraries (WebView2 on Windows, WebKit on macOS, and WebKitGTK on Linux) for ultra-small binaries; why ship an entire web browser with your application when there's a perfectly usable one already installed on the system?

## Philosophy

### Security First
In today's world, every honest threat model assumes that the user's device has already been compromised. It puts app developers in a complicated situation because if the device is already at risk, how can the software be trusted?

Defense in depth is the approach we've taken. We want you to be able to take every precaution possible to minimize the surface area you present to attackers. Millennium lets you choose which API endpoints to ship, whether or not you want a localhost server built into your app, and it even randomizes function handles at runtime. These and other techniques form a secure baseline that empowers you and your users.

Slowing down attackers by making static attacks crushingly difficult and isolating systems from one another is the name of the game; if you are coming from the Electron ecosystem — rest assured — by default, Millennium only ships a single bundled binary, no ASAR files.

By building Millennium with security as a guiding force, we give you every opportunity to take a proactive security posture.

### Polyglots, not Silos
Most contemporary frameworks use a single language paradigm and are trapped in a bubble of knowledge and idiom. It can work well for some niche applications, but it also fosters a kind of tribalism.

This can be seen in the way that the React, Angular, and Vue development communities huddle on their stacks, ultimately breeding very little cross-pollination. The same situation occurs in the Rust vs. Node vs. C++ battlefields, where hardliners take their stances and refuse to collaborate across communities.

Today, Millennium uses Rust for the backend with C++ bindings in the works, but in the not too distant future, other backends like Go, Nim, Python, C#, etc. will be possible. Since the API is implementable in any language with C interop, full compliance is only a PR away.

### Honest Open Source
None of this would make any sense without a community. Today, software communities are amazing places where people help each other and make awesome things - open source is a large part of that.

Open source means different things to different people, but most will agree that it serves to support freedom. When software doesn't respect your rights, it can seem unfair and potentially compromise your freedoms by operating in unethical ways.

This is why we are proud that FLOSS advocates can build applications with Millennium that are "certifiably" open source and can be included in FSF endorsed GNU/Linux distributions.

## Architecture
Millennium is composed of multiple packages working together to create a finished webview. There are three main components of Millennium: `millennium`, `millennium-runtime`, and `millennium-build`.

### The `millennium` package
The `millennium` package is the glue that binds everything together; it is responsible for all frontend APIs, security functions, asset loading, updating, and most importantly, interfacing with the runtime via a high-level API.

### The `millennium-runtime` package
Millennium Runtimes do most of the low-level work. They provide the low-level APIs to control the window and webview used by Millennium. Runtimes are one of Millennium's most unique traits: you can implement a custom runtime and connect it to any Rust-interoperable webview engine.

By default, Millennium uses `millennium-webview` as its runtime, which uses `millennium-core` for windowing. `millennium-webview` is configured as a runtime via the `millennium-runtime-webview` crate.

### The `millennium-build` package
`millennium-build` wraps your app up into a neat little package. It performs compile-time checks to ensure the right Millennium features are enabled and bundles all assets into one executable.

This executable is not ready for deployment just yet — that is handled by `millennium-bundler`, which creates an OS package (e.g. `.dmg`, `.msi`, `.AppImage`) that will install your application in a proper place and install the required webview backends if not installed.

## Security

:::note
While we take every opportunity to help you harden your application - there are always underlying threats like BIOS attacks, row-hammering, and other operating system vulnerabilities that are constantly being discovered and (in the best cases) responsibly disclosed.

Furthermore, there are many ways that development teams can cut corners and either leak sensitive information or leave doors wide open to any of a range of attacks. Security is a never-ending quest, and your users count on you to keep them safe.

Therefore, we highly recommend that you take some time to consider the security ramifications of everything that your application does, especially in the context of running on the semi-hostile platform of end-user devices.
:::

### No Server Required
Millennium enables you to construct an application that uses web technology for the user interface without requiring you to use a server to communicate with the backend. Even if you used advanced techniques of dynamic imports and offload work to the backend, no traffic can be sniffed on TCP ports or external processes - because they aren't there. This reduces not only the physical and virtual footprint of your final binary by a good deal, but it also reduces the surface area of potential attack vectors by removing them from the equation.

### Rust's Language Features
By turning to the programming language renowned for its memory safety and speed, Millennium simply erases whole classes of conventional attacks. "Use after free" just isn't something that can happen with Millennium.

### Dynamic Ahead-of-Time Compilation
This compilation process happens several times during the bootstrapping phase of a Millennium app. Using our default dynamic Ahead-of-Time compiler, you can generate code references that are unique for every session and are still technically static code units.

### Function Hardening

#### Functional ASLR
Functional Address Space Layout Randomization randomizes function names at runtime and can implement OTP hashing, so no two sessions are ever the same. We propose a novel type of function naming at boot time and optionally after every execution. Using a UID for each function pointer prevents static attacks.

#### Kamikaze Function Injection
This advanced type of FASLR using the Event API endpoint is a promise wrapped in a closure (with randomized handle) that Rust inserts at runtime into the WebView, where its interface is locked within the promise resolution handler and is nullified after execution.

#### Bridge, don't serve
Instead of passing potentially unsafe functions, an event bridge can be used to pass messages and commands to named brokers at each respective side of the application.

#### One Time Pad Tokenization and Hashing
Hashing important messages with an OTP salt, you can encrypt messages between the user interface and the Rust backend.

### System Features

#### Enable/Disable APIs
You choose which API functions are available to the UI and Rust. Disabled APIs will not even ship with your app, which reduces binary size and attack surface.

#### Content Security Policy Management
Preventing unauthorized code execution for websites has long since been "resolved" using CSPs. Millennium can inject CSPs into the `index.html` of the user interface, and when using a localhost server, it will also send these headers to the UI or any other clients that connect with it.

#### Decompilation is Difficult
Because assets get bundled directly into the application, your apps cannot be decompiled easily, as is the case with Electron's ASAR files, which makes reverse engineering your project much more time-intensive and requires specialist training.
