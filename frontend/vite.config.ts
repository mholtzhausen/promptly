import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { viteSingleFile } from "vite-plugin-singlefile";
import { readFileSync, writeFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, resolve } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [
    react(),
    viteSingleFile(),
    {
      name: "strip-module-attr",
      closeBundle() {
        const out = resolve(__dirname, "dist", "index.html");
        let html = readFileSync(out, "utf-8");
        // Module scripts with crossorigin from null origin don't work in WebKitGTK.
        html = html.replace(
          /<script type="module" crossorigin>/g,
          "<script>",
        );
        html = html.replace(/<script type="module" /g, "<script ");
        writeFileSync(out, html, "utf-8");
      },
    },
  ],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    assetsInlineLimit: Number.MAX_SAFE_INTEGER,
    modulePreload: false,
    cssCodeSplit: false,
  },
});