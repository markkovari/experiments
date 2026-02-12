import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5177,
    proxy: {
      '/todos': {
        target: 'http://localhost:3003',
        changeOrigin: true,
      },
    },
  },
});
