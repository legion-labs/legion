import { svelte } from "@sveltejs/vite-plugin-svelte";
import { join } from "path";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";
import sveltePreprocess from "svelte-preprocess";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    tsconfigPaths({
      extensions: [".ts", ".svelte"],
    }),
    svelte({
      preprocess: [sveltePreprocess({ typescript: true })],
    }),
  ],
  resolve: {
    alias: {
      "@": join(__dirname, "src"),
    },
  },
});
