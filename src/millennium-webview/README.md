<div align=center>
	<a href="https://millennium.pyke.io"><img src="https://github.com/pykeio/millennium/raw/main/.github/banner.png" width=1920></a>
	<sub><i>Millennium Icon by <a href="https://github.com/xfaonae">XFaon</a>. *Stats are from <a href="https://github.com/tauri-apps/tauri">Tauri</a> and may not be fully accurate.</i><sub>
	<br /><br />
	<a href="https://github.com/pykeio/millennium/actions"><img alt="GitHub Workflow Status" src="https://img.shields.io/github/workflow/status/pykeio/millennium/Test%20Millennium%20(Rust)?style=flat-square"></a> <a href="https://github.com/pykeio/millennium/actions"><img alt="Audit Status" src="https://img.shields.io/github/workflow/status/pykeio/millennium/Audit?style=flat-square&label=audit"></a> <img alt="Lines of code" src="https://img.shields.io/tokei/lines/github/pykeio/millennium?style=flat-square"> <img alt="GitHub commit activity" src="https://img.shields.io/github/commit-activity/w/pykeio/millennium?style=flat-square"> <img alt="Crates.io" src="https://img.shields.io/crates/d/millennium?style=flat-square">
	<br /><br />
	<hr />
</div>

Millennium is an experimental cross-platform webview framework written in Rust. With Millennium, you can design consistent UI that works across all platforms, using HTML, CSS, and JavaScript.
		
## How It Works
You can interact with native code and perform system-level operations, including reading/writing files & TCP/UDP networking. It leverages modern operating systems' pre-included webview libraries (<img src="https://cdn.jsdelivr.net/gh/devicons/devicon/icons/ubuntu/ubuntu-plain.svg" height=14 /> WebKitGTK, <img src="https://cdn.jsdelivr.net/gh/devicons/devicon/icons/windows8/windows8-original.svg" height=14 /> WebView2, <img src="https://cdn.jsdelivr.net/gh/devicons/devicon/icons/apple/apple-original.svg" height=14 /> WebKit) for smaller, faster, more secure, and less resource-heavy applications compared to Electron. A simple Millennium app can be less than **10 MB** in size and can be reduced further to less than **2 MB**. Millennium apps can launch almost twice as fast as equivalent Electron applications and use as little as __1/4 of the amount of RAM.__

Millennium is a fork of [Tauri](https://tauri.studio/), its [official plugins](https://github.com/tauri-apps/awesome-tauri#plugins), [tao](https://github.com/tauri-apps/tao/), and [wry](https://github.com/tauri-apps/wry). We have merged them all into one repo and made some changes suitable for [Allie Project](https://github.com/allie-project/) and [pyke](https://github.com/pykeio/)'s internal projects.

## The `millennium-webview` crate
Millennium Webview is a cross-platform webview rendering library built for Millennium in Rust that supports all major desktop platforms like Windows, macOS, and Linux.

Learn more about Millennium and how to get started at https://millennium.pyke.io.
