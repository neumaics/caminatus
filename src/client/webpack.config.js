const path = require('path');

module.exports = {
  entry: path.resolve(__dirname, './index.js'),
  module: {
    rules: [
      {
        test: /\.(js)$/,
        exclude: /node_modules/,
        use: ['babel-loader']
      }
    ]
  },
  resolve: {
    extensions: ['*', '.js']
  },
  output: {
    path: path.resolve(__dirname, '../../public'),
    filename: 'bundle.js'
  },
};