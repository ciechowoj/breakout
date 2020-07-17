const CopyWebpackPlugin = require("copy-webpack-plugin");
const merge = require('webpack-merge');
const common = require('./webpack.common.js');

module.exports = merge(common, {
  mode: 'production',
  plugins: [
    new CopyWebpackPlugin(
      [
        {
          from: '.htaccess',
          force: true
        },
        {
          from: '../api/target/release/api',
          force: true
        }
      ])
  ],
});
