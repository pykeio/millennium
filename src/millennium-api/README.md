<div align=center>
	<a href="https://millennium.pyke.io"><img src="https://github.com/pykeio/millennium/raw/master/.github/banner.png" width=1920></a>
	<sub><i>Millennium Icon by <a href="https://github.com/xfaonae">XFaon</a>. *Stats are from <a href="https://github.com/tauri-apps/tauri">Tauri</a> and may not be fully accurate.</i><sub>
	<br /><br />
	<a href="https://github.com/pykeio/millennium/actions"><img alt="GitHub Workflow Status" src="https://img.shields.io/github/workflow/status/pykeio/millennium/Test%20Millennium%20(Rust)?style=flat-square"></a> <img alt="Code Climate maintainability" src="https://img.shields.io/codeclimate/maintainability/pykeio/millennium?label=maintainability&style=flat-square">
	<br /><br />
	<hr />
</div>

Millennium is an experimental cross-platform GUI framework written in Rust. With Millennium, you can design consistent UI that works across all platforms, using HTML, CSS, and JavaScript.

## `@pyke/millennium-api`
This package provides the JavaScript API for [Millennium](https://millennium.pyke.io/) applications. With `@pyke/millennium-api`, you can create windows, execute processes, read files, and more from JavaScript with APIs similar to (but not 100% compatible with) Node.js.

You can also access the Millennium API without this module via `window.Millennium` if `build > withGlobalMillennium` is enabled in your `.millenniumrc`. We highly recommend that you bundle this module with your frontend code for production apps for security.

### Examples & documentation
For examples on API usage, check the [examples in the Millennium repo](https://github.com/pykeio/millennium/tree/master/examples) and the [online docs](https://millennium.pyke.io/).
