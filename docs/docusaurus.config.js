// @ts-check
const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/vsDark');

/** @type {import('@docusaurus/types').Config} */
const config = {
	title: 'Millennium',
	tagline: 'Millennium is an experimental cross-platform GUI framework written in Rust. With Millennium, you can design consistent UI that works across all platforms, using HTML, CSS, and JavaScript.',
	url: 'https://millennium.pyke.io',
	baseUrl: '/',
	onBrokenLinks: 'throw',
	onBrokenMarkdownLinks: 'warn',
	favicon: 'img/favicon.ico',
	organizationName: 'pykeio', // Usually your GitHub org/user name.
	projectName: 'millennium', // Usually your repo name.

	presets: [
		[
			'classic',
			/** @type {import('@docusaurus/preset-classic').Options} */
			({
				docs: {
					path: 'docs',
					sidebarPath: require.resolve('./sidebars.js'),
					editUrl: 'https://github.com/pykeio/millennium/edit/master/docs/',
					versions: {
						current: {
							label: 'current'
						}
					},
					lastVersion: 'current',
					showLastUpdateAuthor: true,
					showLastUpdateTime: true
				},
				blog: {
					showReadingTime: true,
					editUrl: 'https://github.com/facebook/docusaurus/edit/master/docs/'
				},
				theme: {
					customCss: require.resolve('./src/css/custom.css'),
				}
			})
		]
	],
	plugins: ['@millennium/image-loader', '@millennium/tailwind-loader'],
	themeConfig:
		/** @type {import('@docusaurus/preset-classic').ThemeConfig} */
		({
			navbar: {
				title: 'Millennium',
				logo: {
					alt: 'Millennium Logo',
					src: 'img/millennium.png',
				},
				items: [
					{
						type: 'doc',
						docId: 'main/intro',
						position: 'left',
						label: 'Docs'
					},
					{
						to: '/blog',
						label: 'Blog',
						position: 'left'
					},

					{
						href: 'https://github.com/pykeio/millennium',
						position: 'right',
						className: 'header-github-link'
					}
				]
			},
			colorMode: {
				defaultMode: 'dark',
				disableSwitch: false,
				respectPrefersColorScheme: true
			},
			footer: {
				links: [
					{
						title: 'Docs',
						items: [
							{
								label: 'Tutorial',
								to: '/docs/intro',
							},
						],
					},
					{
						title: 'Community',
						items: [
							{
								label: 'Stack Overflow',
								href: 'https://stackoverflow.com/questions/tagged/docusaurus',
							},
							{
								label: 'Discord',
								href: 'https://discordapp.com/invite/docusaurus',
							},
							{
								label: 'Twitter',
								href: 'https://twitter.com/docusaurus',
							},
						],
					},
					{
						title: 'More',
						items: [
							{
								label: 'Blog',
								to: '/blog',
							},
							{
								label: 'GitHub',
								href: 'https://github.com/facebook/docusaurus',
							},
						],
					},
				],
				logo: {
					alt: 'Millennium Logo',
					src: 'img/millennium.png'
				},
				copyright: `Copyright © ${new Date().getFullYear()} <a href="https://pyke.io" target="_blank">pyke.io</a>, made with ❤️`,
			},
			image: 'img/millennium.png',
			prism: {
				theme: lightCodeTheme,
				darkTheme: darkCodeTheme
			}
		}),
	themes: [
		[
			require.resolve('@easyops-cn/docusaurus-search-local'),
			{
				hashed: true,
				removeDefaultStopWordFilter: true,
				highlightSearchTermsOnTargetPage: true
			}
		]
	]
};

module.exports = config;
