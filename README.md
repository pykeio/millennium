<div align=center>
	<a href="https://millennium.pyke.io"><img src="https://github.com/pykeio/millennium/raw/master/banner.png" width=1920></a>
	<sub><i>Millennium Icon by <a href="https://github.com/xfaonae">XFaon</a>. *Stats are from <a href="https://github.com/tauri-apps/tauri">Tauri</a> and may not be fully accurate.</i><sub>
	<br /><br />
	<a href="https://github.com/pykeio/millennium/actions"><img alt="GitHub Workflow Status" src="https://img.shields.io/github/workflow/status/pykeio/millennium/Test%20Millennium%20(Rust)?style=flat-square"></a> <img alt="Code Climate maintainability" src="https://img.shields.io/codeclimate/maintainability/pykeio/millennium?label=maintainability&style=flat-square">
	<br /><br />
	<hr />
</div>

Millennium is an experimental cross-platform GUI framework written in Rust. With Millennium, you can design consistent UI that works across all platforms, using HTML, CSS, and JavaScript.
		
## How It Works
You can interact with native code and perform system-level operations, including reading/writing files & TCP/UDP networking. It leverages modern operating systems' pre-included webview libraries (<img src="https://cdn.jsdelivr.net/gh/devicons/devicon/icons/ubuntu/ubuntu-plain.svg" height=14 /> WebKitGTK, <img src="https://cdn.jsdelivr.net/gh/devicons/devicon/icons/windows8/windows8-original.svg" height=14 /> WebView2, <img src="https://cdn.jsdelivr.net/gh/devicons/devicon/icons/apple/apple-original.svg" height=14 /> WebKit) for smaller, faster, more secure, and less resource-heavy applications compared to Electron. A simple Millennium app can be less than **10 MB** in size and can be reduced further to less than **2 MB**. Millennium apps launch almost twice as fast as equivalent Electron applications and use as little as __1/4 of the amount of RAM.__

Millennium is a fork of [Tauri](https://tauri.studio/), its [official plugins](https://github.com/tauri-apps/awesome-tauri#plugins), [tao](https://github.com/tauri-apps/tao/), and [wry](https://github.com/tauri-apps/wry). We have merged them all into one repo and made some changes suitable for [Allie Project](https://github.com/allie-project/) and [pyke](https://github.com/pykeio/)'s internal projects. You should probably use Tauri for your projects as it has a much larger community.

## Upstream

* [**tao**](https://github.com/tauri-apps/tao) @ [`11dac10`](https://github.com/tauri-apps/tao/tree/11dac10241330c30aae660a2621d43ee5eb3775d/)
* [**wry**](https://github.com/tauri-apps/wry) @ [`742a8cb`](https://github.com/tauri-apps/tao/tree/742a8cb87802b2964b2e888d73347777b9164f77/)
* [**tauri**](https://github.com/tauri-apps/tauri) @ [`0f15589`](https://github.com/tauri-apps/tauri/tree/0f1558980a0fb1d6c042988e173047f0590b6574/)
