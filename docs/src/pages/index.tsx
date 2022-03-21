import React, { useEffect } from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import { useColorMode } from '@docusaurus/theme-common';
import { ResponsiveBar } from '@nivo/bar';

function Homepage() {
	const { isDarkTheme } = useColorMode();
	return <div id="tailwind">
		<div className="absolute z-20 top-0 inset-x-0 flex justify-center overflow-hidden pointer-events-none" style={{ userSelect: 'none' }}>
			<div className="w-[108rem] flex-none flex justify-end">
				{isDarkTheme
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
					</div>
				</div>
				<img src={useBaseUrl('/img/Untitled.png')} className='pointer-events-none' draggable={false} style={{ userSelect: 'none', WebkitUserDrag: 'none' } as any} />
			</div>
		</div>
		<div className='py-16 overflow-hidden diagonal-box'>
			<div className='diagonal-content max-w-2xl mx-auto px-4 md:px-6 lg:px-8 lg:max-w-screen-xl'>
				<div className='max-w-screen-xl mx-auto pt-6 px-4 md:px-6 lg:px-8'>
					<div className='max-w-4xl mx-auto text-center'>
						<h2 className='text-3xl leading-9 font-extrabold md:text-4xl md:leading-10'>Light as a feather</h2>
						<p className='mt-4 max-w-2xl text-xl leading-7 lg:mx-auto dark:text-gray-500'>
							Millennium utilizes the webview framework that comes included with operating systems for ultra-lightweight binaries.
						</p>
					</div>
					<div className='py-8 w-full mx-auto px-4 md:px-6 lg:max-w-screen-lg lg:px-8' style={{ height: '270px', userSelect: 'none' }}>
						<ResponsiveBar
							data={[
								{ framework: 'Millennium', binary: 2.09, color: '#51c5eb' },
								{ framework: 'Electron', binary: 185.95, color: '#605b94' },
								{ framework: 'NW.js', binary: 223.82, color: '#605b94' },
							]}
							keys={[ 'binary' ]}
							valueFormat={v => `${v} MB`}
							indexBy='framework'
							layout='horizontal'
							isInteractive={false}
							borderRadius={6}
							motionConfig='gentle'
							margin={{ top: 50, right: 20, bottom: 50, left: 80 }}
							padding={0.3}
							valueScale={{ type: 'linear' }}
							indexScale={{ type: 'band', round: true }}
							colors={[ '#605b94', '#51c5eb' ]}
							colorBy='indexValue'
							axisBottom={{
								tickSize: 5,
								tickPadding: 5,
								tickRotation: 0,
								legend: 'binary size (MB)',
								legendPosition: 'middle',
								legendOffset: 40,
								tickValues: [ 25, 50, 75, 100, 125, 150, 175, 200, 225, 250 ]
							}}
							labelSkipWidth={12}
							labelTextColor={'#fff'}
							theme={{
								axis: {
									ticks: {
										text: {
											fill: isDarkTheme ? 'rgba(255,255,255,0.35)' : 'rgba(0,0,0,0.65)',
											fontWeight: 'bold',
											fontSize: '13px'
										}
									},
									legend: {
										text: {
											fill: isDarkTheme ? 'rgba(255,255,255,0.3)' : 'rgba(0,0,0,0.6)',
										}
									}
								},
								grid: {
									line: {
										stroke: isDarkTheme ? 'rgba(255,255,255,0.3)' : 'rgba(0,0,0,0.3)',
									}
								}
							}}
						/>
					</div>
					<p className='py-8 text-center text-gray-500'>No, that's not an error, it really is just that small.</p>
				</div>
			</div>
		</div>
	</div>
}

export default function Home(): JSX.Element {
	const { siteConfig } = useDocusaurusContext();

	return (
		<Layout title={`Hello from ${siteConfig.title}`} description="Description will go into a meta tag in <head />">
			<Homepage />
		</Layout>
	);
}
