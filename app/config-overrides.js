const webpack = require('webpack'); 
module.exports = function override(config) { 
		const fallback = config.resolve.fallback || {}; 
		Object.assign(fallback, { 
    	process: require.resolve("process/browser"),
          stream: require.resolve("stream-browserify"),
          util: require.resolve("util"),
          buffer: require.resolve("buffer"),
          asset: require.resolve("assert"),
          "zlib": false,
      }) 
   config.resolve.fallback = fallback; 
   config.plugins = (config.plugins || []).concat([ 
   	new webpack.ProvidePlugin({ 
    	process: 'process/browser', 
      Buffer: ['buffer', 'Buffer'] 
    }) 
   ]) 
   return config; }