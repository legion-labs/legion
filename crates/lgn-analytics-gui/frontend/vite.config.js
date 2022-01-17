// @ts-check

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

// https://vitejs.dev/config/
export default async function () {
  const viteTsProto = await import("vite-plugin-ts-proto").then(
    ({ default: viteTsProto }) =>
      viteTsProto({
        modules: [{ name: "@lgn/proto-telemetry", glob: "*.proto" }],
      })
  );

  return defineConfig({
    plugins: [
      tsconfigPaths({
        extensions: [".ts", ".svelte"],
      }),
      svelte(),
      viteTsProto,
    ],
  });
}
