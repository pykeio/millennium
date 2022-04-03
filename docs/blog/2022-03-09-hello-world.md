---
slug: hello-world
title: Hello, world! Millennium is here
authors: [carson]
tags: []
---

Millennium v1.0.0 beta 1, the first release of Millennium, is here! This is a beta version, so expect some bugs, but for the most part, it should be stable.

Some upcoming features for future versions include:

### Visual Studio Code extension (**stable**)
- The VSCode extension provides autocompletion & IntelliSense in `.millenniumrc` config files.
- With the addition of CLI tools, we'll add CLI commands such as `millennium build` to the Command Palette.

### Official plugins, including YubiKey integration, Vibrancy/Acrylic, and Web Sockets (**stable**)
- We'll be porting some official Tauri plugins to Millennium in the stable release.
- You can expect to see at least the plugins mentioned above ported for the stable release. Not all plugins may be available until v1.1 or v1.2.

### C++ bindings (**v1.1**)
- C++ is the most similar language to Rust, so it just seemed natural to add C++ bindings 🙂
- You'll still need to build a small Rust wrapper because Millennium bakes config & features into the binary at compile-time, but it requires minimal interaction.

### Permissionless APIs (**v1.1**)
- Currently, APIs that require permissions (besides notifications) do not work across all platforms and get automatically denied by the webview.
- We hope to have a feature to allow/disallow certain permissions without user interaction.
