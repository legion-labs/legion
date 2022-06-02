// @ts-check
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

import viteTsProto from "@lgn/vite-plugin-ts-proto";

process.env.VITE_CONSOLE_LOG_LEVEL = "debug";

// https://vitejs.dev/config/
export default defineConfig(() => {
  return {
    plugins: [
      tsconfigPaths({
        extensions: [".ts", ".svelte"],
      }),
      // The `!!` trick is necessary here or the whole expression
      // will return `undefined` which make hot `true`...
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      svelte({ hot: !!process.env.DEV && !process.env.VITEST }),
      viteTsProto({
        modules: [
          { name: "@lgn/proto-log-stream", glob: "*.proto" },
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
  };
});
