---
slug: hello-world
title: Hello, world! Millennium is here
authors: [carson]
tags: []
---

Millennium v1.0.0 beta 1, the first release of Millennium, is here! This is a beta version, so expect some bugs, but for the most part, it should be stable.
The API is guaranteed to be stable, with only additions & deprecations until v2.0.

Some upcoming features for future versions include:

### CLI tools (**stable**)
- `millennium init` creates a new Millennium project from a template, similar to `create-react-app`;
- `millennium build` builds your application for production;
- `millennium dev` allows for rapid development, rebuilding your Rust code when it's changed, and supporting hot reloading (if your tooling supports it)

### Visual Studio Code extension (**stable**)
- The VSCode extension provides autocompletion & IntelliSense in `.millenniumrc` config files.
- With the addition of CLI tools, we'll add CLI commands such as `millennium build` to the Command Palette.

### Official plugins, including YubiKey integration, Vibrancy/Acrylic, and Web Sockets (**v1.1**)
- We'll be porting some official Tauri plugins to Millennium in release v1.1.
- You can expect to see at least the plugins mentioned above ported for v1.1. Not all plugins may be available until v1.2 or v2.0.

### C++ bindings (**v1.1**)
- C++ is the most similar language to Rust, so it just seemed natural to add C++ bindings 🙂
- You'll still need to touch a small amount of Rust due to Millennium baking config & features into the binary at compile-time, but we hope that it'll be auto-generated, and you'd only need to run a single command.
