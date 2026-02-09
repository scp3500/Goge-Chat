import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
    clearScreen: false,
    server: {
        port: 1430,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                protocol: "ws",
                host,
                port: 1431,
            }
            : undefined,
        watch: {
            ignored: ["**/src-tauri/**"],
        },
        // 优化开发体验：禁用缓存，避免 Live2D 文件更新不及时
        headers: {
            'Cache-Control': 'no-store',
        },
    },
    build: {
        minify: 'terser',
        terserOptions: {
            compress: {
                drop_console: true,
                drop_debugger: true,
            },
        },
        rollupOptions: {
            output: {
                manualChunks(id) {
                    if (id.includes('node_modules')) {
                        if (id.includes('pixi') || id.includes('live2d')) {
                            return 'vendor-pixi';
                        }
                        return 'vendor';
                    }
                }
            }
        },
        chunkSizeWarningLimit: 1500
    }
}));
