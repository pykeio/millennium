// @ts-check
const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/vsDark');

/** @type {import('@docusaurus/types').Config} */
const config = {
	title: 'Millennium',
	tagline: 'Millennium is a cross-platform GUI framework written in Rust. With Millennium, you can design consistent UI that works across all platforms, using HTML, CSS, and JavaScript.',
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
					editUrl: 'https://github.com/pykeio/millennium/edit/main/docs/',
					versions: {
						current: {
							label: 'current'
						}
					},
					lastVersion: 'current',
					showLastUpdateAuthor: true,
					showLastUpdateTime: true
				},
				sitemap: {
					changefreq: 'weekly',
					priority: 0.5
				},
				blog: {
					showReadingTime: true,
					editUrl: 'https://github.com/pykeio/millennium/edit/main/docs/'
				},
				theme: {
					customCss: require.resolve('./src/css/custom.css'),
				}
			})
		]
	],
	plugins: [
		'@millennium/image-loader',
		'@millennium/tailwind-loader',
		[
			'@docusaurus/plugin-pwa',
			{
				offlineModeActivationStrategies: [
					'appInstalled',
					'standalone',
					'queryString'
				],
				pwaHead: [
					{
						tagName: 'link',
						rel: 'icon',
						href: '/img/millennium.png'
					},
					{
						tagName: 'link',
						rel: 'apple-touch-icon',
						href: '/img/app-icon.png'
					},
					{
						tagName: 'link',
						rel: 'manifest',
						href: '/manifest.json'
					},
					{
						tagName: 'meta',
						name: 'theme-color',
						content: '#605b94'
					}
				]
			}
		]
	],
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
						href: 'https://discord.gg/CETPevXFgD',
						position: 'right',
						className: 'header-discord-link',
						alt: 'Discord server'
					},
					{
						href: 'https://github.com/pykeio/millennium',
						position: 'right',
						className: 'header-github-link',
						alt: 'GitHub repository'
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
								label: 'Architecture',
								to: '/docs/main/intro',
							},
							{
								label: 'Tutorial',
								to: '/docs/main/your-first-app/prerequisites',
							},
						],
					},
					{
						title: 'Community',
						items: [
							{
								label: 'GitHub Discussions',
								href: 'https://github.com/pykeio/millennium/discussions',
							},
							{
								label: 'Discord',
								href: 'https://discord.gg/CETPevXFgD',
							},
							{
								label: 'Twitter',
								href: 'https://twitter.com/pyke_io',
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
								href: 'https://github.com/pykeio/millennium',
							},
						],
					},
				],
				logo: {
					alt: 'Millennium Logo',
					src: 'img/millennium.png',
					width: 75
				},
				copyright: `Copyright © ${new Date().getFullYear()} <a href="https://pyke.io" target="_blank">pyke.io</a>, made with ❤️`,
			},
			image: 'img/banner.png',
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
