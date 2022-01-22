// @ts-check

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";
import viteTsProto from "vite-plugin-ts-proto";
import viteWasmPack from "vite-plugin-wasm";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    tsconfigPaths({
      extensions: [".ts", ".svelte"],
    }),
    svelte(),
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
  ],
});
