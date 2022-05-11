import React, { useEffect } from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import { useColorMode } from '@docusaurus/theme-common';
import { ResponsiveBar } from '@nivo/bar';

function Homepage() {
	const { colorMode } = useColorMode();
	return <div id="tailwind z-20">
		<div className="absolute z-0 top-0 inset-x-0 flex justify-center overflow-hidden pointer-events-none" style={{ userSelect: 'none' }}>
			<div className="w-[108rem] flex-none flex justify-end">
				{colorMode === 'dark'
					? <picture>
						<source srcSet={useBaseUrl('/img/tailwind-bg-dark.avif')} type="image/avif" />
						<img src={useBaseUrl('/img/tailwind-bg-dark.png')} alt="" className="w-[90rem] flex-none max-w-none" />
					</picture>
					: <picture>
						<source srcSet={useBaseUrl('/img/tailwind-bg-light.avif')} type="image/avif" />
						<img src={useBaseUrl('/img/tailwind-bg-light.png')} alt="" className="w-[71.75rem] flex-none max-w-none" />
					</picture>
				}
			</div>
		</div>
		<div className='lg:pt-32 py-16 overflow-hidden'>
			<div className='relative max-w-3xl mx-auto px-4 md:px-6 lg:px-8 lg:max-w-screen-xl'>
				<div className='max-w-screen-xl mx-auto px-4 md:px-6 lg:px-8'>
					<div className='max-w-4xl mx-auto text-center'>
						<h1 className='font-extrabold text-4xl sm:text-5xl lg:text-6xl tracking-tight text-center'>
							Build fast &amp; secure cross-platform web-based UIs
						</h1>
						<div className='py-16 flex flex-col items-center'>
							<div className='flex flex-row flex-wrap w-fit justify-center items-center'>
								<Link href='/docs/main/intro' className='w-fit bg-purple-500 hover:bg-purple-700 m-3 hover:text-purple-300 hover:no-underline shadow-xl text-white font-bold text-2xl py-4 px-8 rounded-full'>
									Learn more
								</Link>
								<Link href='/docs/main/your-first-app/prerequisites' className='w-fit bg-blue-500 hover:bg-blue-700 m-3 hover:text-blue-300 hover:no-underline shadow-xl text-white font-bold text-2xl py-4 px-8 rounded-full'>
									Get Started
								</Link>
							</div>
							<pre className='w-fit my-8'>npm init millennium my-app</pre>
						</div>
					</div>
				</div>
			</div>
		</div>
		<div className='py-16 overflow-hidden diagonal-box'>
			<div className='diagonal-content max-w-2xl mx-auto px-4 md:px-6 lg:px-8 lg:max-w-screen-xl'>
				<div className='max-w-screen-xl pt-6 md:px-6 lg:px-8'>
					<div className='max-w-4xl mx-auto text-center'>
						<h2 className='text-3xl leading-9 font-extrabold md:text-4xl md:leading-10'>Light as a feather</h2>
						<p className='mt-4 max-w-2xl text-xl leading-7 lg:mx-auto dark:text-gray-400'>
							Millennium utilizes the webview framework that comes included with operating systems for ultra-lightweight binaries.
						</p>
					</div>
					<div className='py-8 w-full mx-auto md:px-6 lg:max-w-screen-lg lg:px-8' style={{ height: '550px', maxWidth: '500px', userSelect: 'none' }}>
						<ResponsiveBar
							data={[
								{ framework: 'NW.js', binary: 223.82 },
								{ framework: 'Electron', binary: 185.95 },
								{ framework: 'Millennium', binary: 1.69 },
							]}
							keys={[ 'binary' ]}
							valueFormat={v => `${v} MB`}
							indexBy='framework'
							isInteractive={false}
							borderRadius={6}
							motionConfig='gentle'
							margin={{ top: 50, right: 20, bottom: 50, left: 50 }}
							padding={0.3}
							valueScale={{ type: 'linear' }}
							indexScale={{ type: 'band', round: true }}
							colors={[ '#d45d59', '#6dd678', '#5e9ddb' ]}
							colorBy='indexValue'
							labelTextColor={colorMode === 'dark' ? 'rgba(255,255,255,0.9)' : 'rgba(0,0,0,0.65)'}
							theme={{
								labels: {
									text: {
										textShadow: '0px 0px 3px rgba(0,0,0,0.25)',
										transform: 'translateY(-10px)'
									}
								},
								axis: {
									ticks: {
										text: {
											fill: colorMode === 'dark' ? 'rgba(255,255,255,0.55)' : 'rgba(0,0,0,0.65)',
											fontWeight: '600',
											fontSize: '14px'
										}
									},
									legend: {
										text: {
											fill: colorMode === 'dark' ? 'rgba(255,255,255,0.3)' : 'rgba(0,0,0,0.6)',
											fontSize: '13px'
										}
									}
								},
								grid: {
									line: {
										stroke: colorMode === 'dark' ? 'rgba(255,255,255,0.3)' : 'rgba(0,0,0,0.3)',
									}
								}
							}}
						/>
					</div>
				</div>
			</div>
		</div>
	</div>
}

export default function Home(): JSX.Element {
	const { siteConfig } = useDocusaurusContext();

	return (
		<Layout title={`Hello from ${siteConfig.title}`} description="Millennium is a cross-platform GUI framework written in Rust. With Millennium, you can design consistent UI that works across all platforms, using HTML, CSS, and JavaScript.">
			<Homepage />
		</Layout>
	);
}
