import react from '@vitejs/plugin-react'
import path from 'path'
import { defineConfig } from 'vite'
import { VitePWA } from 'vite-plugin-pwa'

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const isDev = mode === 'development'
  const isFast = process.env.VITE_FAST_BUILD === 'true'

  return {
    plugins: [
      react({
        fastRefresh: isDev,
        // Skip react devtools in fast mode
        babel: isFast ? { plugins: [] } : undefined
      }),
      // Only enable PWA in production
      ...(isDev ? [] : [
        VitePWA({
          registerType: 'autoUpdate',
          includeAssets: ['favicon.ico', 'apple-touch-icon.png', 'masked-icon.svg'],
          manifest: {
            name: 'Uveddi Interactive Reports',
            short_name: 'Uveddi Reports',
            description: 'Interactive architectural analysis reports',
            theme_color: '#1976d2',
            background_color: '#ffffff',
            display: 'standalone',
            orientation: 'portrait',
            scope: '/',
            start_url: '/',
            icons: [
              {
                src: 'pwa-192x192.png',
                sizes: '192x192',
                type: 'image/png'
              },
              {
                src: 'pwa-512x512.png',
                sizes: '512x512',
                type: 'image/png'
              }
            ]
          },
          workbox: {
            globPatterns: ['**/*.{js,css,html,ico,png,svg}'],
            runtimeCaching: [
              {
                urlPattern: /^\/api\//,
                handler: 'NetworkFirst',
                options: {
                  cacheName: 'api-cache',
                  expiration: {
                    maxEntries: 100,
                    maxAgeSeconds: 60 * 60 * 24 // 24 hours
                  }
                }
              }
            ]
          }
        })
      ])
    ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/hooks': path.resolve(__dirname, './src/hooks'),
      '@/services': path.resolve(__dirname, './src/services'),
      '@/types': path.resolve(__dirname, './src/types'),
      '@/utils': path.resolve(__dirname, './src/utils')
    }
  },
    server: {
      port: 8001,
      hmr: !isFast, // Disable HMR in fast mode
      proxy: {
        '/api': {
          target: 'http://localhost:8000',
          changeOrigin: true
        },
        '/health': {
          target: 'http://localhost:8000',
          changeOrigin: true
        },
        '/metrics': {
          target: 'http://localhost:8000',
          changeOrigin: true
        }
      }
    },
    build: {
      outDir: 'dist',
      sourcemap: isDev || !isFast, // Disable sourcemaps in fast production builds
      minify: isDev ? false : 'esbuild', // Use faster esbuild minifier
      target: isDev ? 'esnext' : 'es2020',
      rollupOptions: {
        output: {
          // Aggressive chunking for better caching
          manualChunks: isDev ? undefined : {
            'vendor-react': ['react', 'react-dom', 'react-router-dom'],
            'vendor-ui': ['@mui/material', '@mui/icons-material', '@emotion/react', '@emotion/styled'],
            'vendor-charts': ['chart.js', 'react-chartjs-2', 'recharts'],
            'vendor-graphs': ['cytoscape', 'react-cytoscapejs', 'd3'],
            'vendor-diagrams': ['mermaid', 'html2canvas', 'jspdf'],
            'vendor-utils': ['lodash', 'date-fns', 'fuse.js'],
            'vendor-grid': ['react-grid-layout', 'react-virtualized', 'react-window']
          }
        }
      },
      chunkSizeWarningLimit: 1000 // Increase threshold for chunk size warnings
    },
    optimizeDeps: {
      include: [
        'react',
        'react-dom',
        'react-router-dom',
        'cytoscape',
        '@mui/material',
        '@emotion/react',
        '@emotion/styled'
      ],
      exclude: isFast ? ['@mui/icons-material'] : [] // Exclude heavy deps in fast mode
    },
    esbuild: {
      // Drop console and debugger in production
      drop: isDev ? [] : ['console', 'debugger'],
      // Use faster transformations
      target: isDev ? 'esnext' : 'es2020'
    }
  }
})