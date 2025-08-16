import { fileURLToPath } from "node:url";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite";

export default defineConfig({
    plugins: [
        UnoCSS(fileURLToPath(new URL("./uno.config.ts", import.meta.url))),
    ],
});
