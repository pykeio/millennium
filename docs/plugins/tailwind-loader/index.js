module.exports = () => /** @type {import('@docusaurus/types').Plugin} */({
	name: 'tailwind-loader',
	injectHtmlTags() {
		return {
			headTags: [
				{
					tagName: 'script',
					innerHTML: `
const observer = new MutationObserver(mutations => {
	mutations.forEach(mutation => {
		if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {
			const html = document.querySelector('html');
			const isDarkTheme = html.getAttribute('data-theme') === 'dark';
			if (isDarkTheme)
				html.classList.add('dark');
			else
				html.classList.remove('dark');
		}
	});
});
observer.observe(document.querySelector('html'), {
	attributes: true,
	attributeFilter: [ 'data-theme' ]
});

window.addEventListener('DOMContentLoaded', () => {
	document.addEventListener('scroll', () => {
		const header = document.getElementsByClassName('navbar')[0];
		if (!header)
			return;

		if (window.scrollY > 0)
			header.classList.add('scroll');
		else
			header.classList.remove('scroll');
	});
});
`
				},
				{
					tagName: 'link',
					attributes: {
						rel: 'stylesheet',
						href: 'https://parcel.pyke.io/v2/cdn/3rdparty/inter-font/3.18/inter.css',
						integrity: 'sha384-hDHjTEEMh9Rupxhm+TvGxKQj+LTqilUq6+4l4ySmDEVALF2jdnHDEPX9i61iT3zF',
						crossorigin: 'anonymous'
					}
				}
			]
		};
	},
	configurePostCss(postcssOptions) {
		postcssOptions.plugins.push(
			require('postcss-import'),
			require('tailwindcss'),
			require('postcss-preset-env')({
				autoprefixer: {
					flexbox: 'no-2009'
				},
				stage: 4
			})
		);
		return postcssOptions;
	}
})
