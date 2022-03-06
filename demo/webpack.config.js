const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./src/index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin({
      patterns: ['public/']
    })
  ],
  watchOptions: {
    poll: 3000
  },
  experiments: {
    asyncWebAssembly: true
  }
};
