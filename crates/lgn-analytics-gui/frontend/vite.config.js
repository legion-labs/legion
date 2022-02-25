// @ts-check

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";
import viteTsProto from "@lgn/vite-plugin-ts-proto";
import path from "path";
import { fileURLToPath } from "url";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
const dirname = path.dirname(fileURLToPath(import.meta.url));

const githubSpaHack = `
<!-- Start Single Page Apps for GitHub Pages -->
<script type="text/javascript">
  // Single Page Apps for GitHub Pages
  // MIT License
  // https://github.com/rafgraph/spa-github-pages
  // This script checks to see if a redirect is present in the query string,
  // converts it back into the correct url and adds it to the
  // browser's history using window.history.replaceState(...),
  // which won't cause the browser to attempt to load the new url.
  // When the single page app is loaded further down in this file,
  // the correct url will be waiting in the browser's history for
  // the single page app to route accordingly.
  (function (l) {
    if (l.search[1] === "/") {
      var decoded = l.search
        .slice(1)
        .split("&")
        .map(function (s) {
          return s.replace(/~and~/g, "&");
        })
        .join("?");
      window.history.replaceState(
        null,
        null,
        l.pathname.slice(0, -1) + decoded + l.hash
      );
    }
  })(window.location);
</script>
<!-- End Single Page Apps for GitHub Pages -->
`;

const shouldInjectSpaHack = process.env.GITHUB_PAGES === "true";

/** @param {Record<string, string>} env */
function htmlPlugin(env) {
  return {
    name: "html-transform",
    transformIndexHtml: {
      transform: (/** @type {string} */ html) =>
        html.replace(/%(.*?)%/g, (match, p1) => env[p1] ?? match),
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    rollupOptions: {
      input: {
        main: path.resolve(dirname, "index.html"),
        ...(shouldInjectSpaHack
          ? { notFound: path.resolve(dirname, "404.html") }
          : {}),
      },
    },
  },
  plugins: [
    tsconfigPaths({
      extensions: [".ts", ".svelte"],
    }),
    svelte(),
    viteTsProto({
      modules: [{ name: "@lgn/proto-telemetry", glob: "*.proto" }],
    }),
    htmlPlugin({ INJECTED_SCRIPT: shouldInjectSpaHack ? githubSpaHack : "" }),
  ],
});
