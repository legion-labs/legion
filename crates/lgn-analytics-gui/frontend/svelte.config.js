// @ts-check

import adapter from "@sveltejs/adapter-static";
import preprocess from "svelte-preprocess";
import tsconfigPaths from "vite-tsconfig-paths";
import viteTsProto from "@lgn/vite-plugin-ts-proto";

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: preprocess({
    postcss: true,
    typescript: true,
  }),
  kit: {
    adapter: adapter(),
    vite: {
      plugins: [
        tsconfigPaths({
          extensions: [".ts", ".svelte"],
        }),
        viteTsProto({
          modules: [{ name: "@lgn/proto-telemetry", glob: "*.proto" }],
        }),
      ],
    },
  },
};
