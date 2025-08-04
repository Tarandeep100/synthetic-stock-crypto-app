/** @type {import('next').NextConfig} */
const nextConfig = {
  // Enable hot reloading in development
  reactStrictMode: true,
  
  // Images configuration
  images: {
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'localhost',
      },
    ],
  },
  
  // Webpack configuration optimized for speed
  webpack: (config, { dev, isServer }) => {
    // Original blockchain/wallet config
    config.resolve.fallback = { 
      fs: false, 
      net: false, 
      tls: false,
      crypto: false,
      stream: false,
      url: false,
      zlib: false,
      http: false,
      https: false,
      assert: false,
      os: false,
      path: false
    };
    
    config.externals.push('pino-pretty', 'lokijs', 'encoding');
    
    if (dev) {
      // Faster file watching
      config.watchOptions = {
        poll: 1000,
        aggregateTimeout: 300,
        ignored: /node_modules/,
      };
    }
    
    return config;
  },
}

module.exports = nextConfig