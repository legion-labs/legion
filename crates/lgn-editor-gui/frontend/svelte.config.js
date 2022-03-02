// @ts-check

import adapter from "@sveltejs/adapter-static";
import preprocess from "svelte-preprocess";
import viteConfig from "./vite.config.js";

/** @type {import("@sveltejs/kit").Config} */
export default {
  preprocess: preprocess({
    postcss: true,
    typescript: true,
  }),
  kit: {
    adapter: adapter({
      pages: "dist",
      assets: "dist",
      fallback: "index.html",
    }),
    vite: viteConfig,
  },
};
