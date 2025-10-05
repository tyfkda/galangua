const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "production",
  experiments: {
    asyncWebAssembly: true,
  },
  module: {
    rules: [
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
      },
    ],
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        { from: path.resolve(__dirname, 'index.html'), to: '' },
        { from: path.resolve(__dirname, 'default.css'), to: '' },
        { from: path.resolve(__dirname, 'assets/**/*'), to: '' },
      ],
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ],
  performance: {
    hints: 'warning', // or "error" or false
    maxAssetSize: 512000, // Increase limit to 500 KB
    maxEntrypointSize: 512000, // Increase entry point limit
  },
  devServer: {
    static: path.resolve(__dirname, 'dist'),
    port: 8080,
  },
  resolve: {
    extensions: ['.js', '.wasm'],
  },
};
