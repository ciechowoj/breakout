const CopyWebpackPlugin = require("copy-webpack-plugin");
const WebpackSynchronizableShellPlugin = require('webpack-synchronizable-shell-plugin');
const merge = require('webpack-merge');
const common = require('./webpack.common.js');

module.exports = merge(common, {
  mode: 'development',
  plugins: [
    new WebpackSynchronizableShellPlugin(
      {
        onBuildStart: {
          scripts: ['cargo build --manifest-path=../api/Cargo.toml'],
          blocking: true,
          parallel: false
        },
      }),
    new CopyWebpackPlugin({
        patterns: [
          {
            from: '.htaccess',
            force: true,
            transform(content, absoluteFrom) {
              let regex = /(RewriteCond.*off\n?)|(RewriteRule.*https.*\{REQUEST_URI\}\s\[L,R=301]\n?)/gi
              return content.toString().replace(regex, '');
            }
          },
          {
            from: '../api/target/debug/api',
            force: true
          }
    ]})
  ],
});
