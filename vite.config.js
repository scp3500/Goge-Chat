import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
    plugins: [vue()],

    clearScreen: false,
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                protocol: "ws",
                host,
                port: 1421,
            }
            : undefined,
        watch: {
            // 3. 更加激进的忽略策略：忽略所有以 .db 开头的文件及其产生的临时后缀 [cite: 2026-01-19]
            ignored: [
                "**/src-tauri/**",
                "**/*.db*",
                "**/*.db-journal",
                "**/alice_data.db*"
            ],
        },
        // 优化开发体验：禁用缓存
        headers: {
            'Cache-Control': 'no-store',
        },
    },
    build: {
        // 1. 启用 Terser 压缩 (比默认的 esbuild 压缩率更高)
        minify: 'terser',
        terserOptions: {
            compress: {
                drop_console: true,  // 生产环境移除 console
                drop_debugger: true, // 生产环境移除 debugger
            },
        },
        rollupOptions: {
            input: {
                main: 'index.html'
            },
            output: {
                // 2. 手动分包策略
                manualChunks(id) {
                    if (id.includes('node_modules')) {
                        // 基础框架
                        if (id.includes('vue') || id.includes('pinia')) {
                            return 'vendor-core';
                        }
                        // Markdown 渲染相关
                        if (id.includes('marked') || id.includes('highlight.js') || id.includes('dompurify')) {
                            return 'vendor-markdown';
                        }
                        // 其他第三方库
                        return 'vendor-libs';
                    }
                }
            }
        },
        // 3. 提高警告阈值，避免分包警告
        chunkSizeWarningLimit: 1000
    }
}));