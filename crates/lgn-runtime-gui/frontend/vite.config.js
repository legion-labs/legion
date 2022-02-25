// @ts-check

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";
import viteTsProto from "@lgn/vite-plugin-ts-proto";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    tsconfigPaths({
      extensions: [".ts", ".svelte"],
    }),
    // The `!!` trick is necesary here or the whole expression
    // will return `undefined` which make hot `true`...
    svelte({ hot: !!process.env.DEV && !process.env.VITEST }),
    viteTsProto({
      modules: [
        { name: "@lgn/proto-runtime", glob: "*.proto" },
        { name: "@lgn/proto-streaming", glob: "*.proto" },
      ],
    }),
  ],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: "tests/setup.ts",
  },
});
