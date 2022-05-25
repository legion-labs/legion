// @ts-check
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "path";
import { fileURLToPath } from "url";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

import { loadAll } from "@lgn/config";
import viteTsProto from "@lgn/vite-plugin-ts-proto";

/** @type {"jsdom"} */
const testEnvironment = "jsdom";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
const dirname = path.dirname(fileURLToPath(import.meta.url));

const plugins = [
  tsconfigPaths({
    extensions: [".ts", ".svelte"],
  }),
  viteTsProto({
    modules: [{ name: "@lgn/proto-telemetry", glob: "*.proto" }],
  }),
];

if ("VITEST" in process.env) {
  plugins.push(...svelte({ hot: false }));
}

// https://vitejs.dev/config/
export default defineConfig(() => {
  loadAll({
    VITE_LEGION_ANALYTICS_ONLINE_AUTHENTICATION_OAUTH_ISSUER_URL:
      "online.authentication.issuer_url",
    VITE_LEGION_ANALYTICS_ONLINE_AUTHENTICATION_OAUTH_CLIENT_ID:
      "online.authentication.client_id",
  });

  return {
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
});
