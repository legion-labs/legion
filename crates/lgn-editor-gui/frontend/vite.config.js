// @ts-check

import tsconfigPaths from "vite-tsconfig-paths";
import viteTsProto from "@lgn/vite-plugin-ts-proto";
import path from "path";
import { svelte } from "@sveltejs/vite-plugin-svelte";
// import viteWasmPack from "@lgn/vite-plugin-wasm";

/** @type {"jsdom"} */
const testEnvironment = "jsdom";

const plugins = [
  tsconfigPaths({
    extensions: [".ts", ".svelte"],
  }),
  viteTsProto({
    modules: [
      { name: "@lgn/proto-editor", glob: "*.proto" },
      { name: "@lgn/proto-streaming", glob: "*.proto" },
    ],
  }),
  // viteWasmPack({
  //   crates: [
  //     {
  //       path: "../../../npm-pkgs/simple-wasm-logger",
  //       packageName: "@lgn/simple-wasm-logger",
  //     },
  //   ],
  //   outDir: "frontend",
  //   quiet: true,
  // }),
];

if (process.env.VITEST) {
  plugins.push(svelte({ hot: false }));
}

// https://vitejs.dev/config/
/** @type {import("vite").UserConfig & import("vitest").UserConfig} */
export default {
  // TODO: Drop this option when vite-tsconfig-paths
  // will work properly with SvelteKit
  resolve: {
    alias: {
      "@/resources": path.resolve("./tests/resources"),
      "@": path.resolve("./src"),
    },
  },
  plugins,
  test: {
    environment: testEnvironment,
    globals: true,
    setupFiles: "tests/setup.ts",
  },
};
