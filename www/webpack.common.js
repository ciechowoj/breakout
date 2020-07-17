const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  plugins: [
    new CopyWebpackPlugin(
      [
        {
          from: 'index.html',
          force: true
        }
      ])
  ],
  performance: {
    hints: false
  }        
};
