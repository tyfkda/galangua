const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

const distPath = path.resolve(__dirname, "dist")

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: distPath,
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        {from: 'index.html', to: distPath},
        {from: 'default.css', to: distPath},
        {from: 'assets/**/*', to: distPath},
      ],
    })
  ],
  experiments: {
    syncWebAssembly: true,
  },
};
