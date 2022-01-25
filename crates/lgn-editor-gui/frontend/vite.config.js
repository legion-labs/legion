// @ts-check

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";
import viteTsProto from "@lgn/vite-plugin-ts-proto";
import viteWasmPack from "@lgn/vite-plugin-wasm";

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
    viteWasmPack({
      root: "../../..",
      crates: { "@lgn/simple-wasm-example": "lgn-simple-wasm-example" },
      quiet: true,
    }),
  ],
});
