// @ts-check

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";
import viteTsProto from "vite-plugin-ts-proto";

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
  ],
});
