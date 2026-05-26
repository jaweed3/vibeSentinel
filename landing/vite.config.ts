import { defineConfig } from "vite";

export default defineConfig({
  base: "/vibeSentinel/",
  build: {
    outDir: "dist",
    assetsInlineLimit: 4096,
  },
});
