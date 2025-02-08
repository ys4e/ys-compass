import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tsconfigPaths from "vite-tsconfig-paths";
import postcss from "./cfg/postcss.config";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    plugins: [
        react(),
        tsconfigPaths()
    ],

    css: { postcss },

    // prevent vite from obscuring rust errors
    clearScreen: false,
    // Tauri expects a fixed port, fail if that port is not available
    server: {
        strictPort: true,
        port: 1420
    },
    // to access the Tauri environment variables set by the CLI with information about the current target
    envPrefix: ['VITE_', 'TAURI_PLATFORM', 'TAURI_ARCH', 'TAURI_FAMILY', 'TAURI_PLATFORM_VERSION', 'TAURI_PLATFORM_TYPE', 'TAURI_DEBUG'],
    build: {
        // Tauri uses Chromium on Windows and WebKit on macOS and Linux
        target: "esnext",
        // produce sourcemaps for debug builds
        sourcemap: !!process.env.TAURI_DEBUG,
    }
}));
