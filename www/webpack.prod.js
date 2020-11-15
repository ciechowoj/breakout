const CopyWebpackPlugin = require("copy-webpack-plugin");
const WebpackSynchronizableShellPlugin = require('webpack-synchronizable-shell-plugin');
const merge = require('webpack-merge');
const common = require('./webpack.common.js');

module.exports = merge(common, {
  mode: 'production',
  plugins: [
    new WebpackSynchronizableShellPlugin(
      {
        onBuildStart: {
          scripts: [
            'cargo build --manifest-path=../api/Cargo.toml --release',
            'wasm-pack build ../'
          ],
          blocking: true,
          parallel: false
        },
      }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: '.htaccess',
          force: true
        },
        {
          from: '../api/target/release/api',
          force: true
        }
    ]})
  ],
});
