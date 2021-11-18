import vue from "@vitejs/plugin-vue";
import { join } from "path";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    tsconfigPaths({
      extensions: [".ts", ".vue"],
    }),
    vue(),
  ],
  resolve: {
    alias: {
      "@": join(__dirname, "src"),
    },
  },
});
