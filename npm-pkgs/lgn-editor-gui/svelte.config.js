// @ts-check
import adapter from "@sveltejs/adapter-static";
import preprocess from "svelte-preprocess";

// TODO: Drop the any
/** @type {import("@sveltejs/kit").Config & any} */
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
  },
};
