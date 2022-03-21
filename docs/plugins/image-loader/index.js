module.exports = () => ({
	name: 'image-loader',
	configureWebpack() {
		return {
			module: {
				rules: [
					{
						test: /\.(png|jpe?g|gif|avif)(\?.*)?$/i,
						exclude: /\.(mdx?|svg)$/i,
						use: [ 'file-loader', { loader: 'image-webpack-loader' } ]
					}
				]
			}
		};
	}
})
