const CopyWebpackPlugin = require('copy-webpack-plugin')
const path = require('path')

module.exports = {
  entry: './src/index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'index.js',
  },
  devtool: 'eval-source-map',
  mode: 'development',
  plugins: [
    new CopyWebpackPlugin({
      patterns: ['public/']
    })
  ],
  watchOptions: {
    poll: 3000
  },
  devServer: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  },
  ignoreWarnings: [
    /Circular dependency between chunks with runtime/
  ],
  experiments: {
    asyncWebAssembly: true
  }
}
