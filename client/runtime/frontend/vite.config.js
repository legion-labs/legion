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
        { name: "@lgn/proto-runtime", glob: "*.proto" },
        { name: "@lgn/proto-streaming", glob: "*.proto" },
      ],
    }),
    viteWasmPack({
      crates: [
        { path: "../../../lib/browser-auth", packageName: "@lgn/browser-auth" },
      ],
      outDir: "frontend",
      quiet: true,
    }),
  ],
});
