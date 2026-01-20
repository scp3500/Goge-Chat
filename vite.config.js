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
    },
}));