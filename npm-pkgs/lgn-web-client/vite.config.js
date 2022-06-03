import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig(() => {
  return {
    // The `!!` trick is necessary here or the whole expression
    // will return `undefined` which make hot `true`...
    plugins: [svelte({ hot: !!process.env.DEV && !process.env.VITEST })],
    test: {
      environment: "jsdom",
      globals: true,
      setupFiles: "tests/setup.ts",
    },
  };
});
