// eslint-disable-next-line @typescript-eslint/no-var-requires
const fs = require("fs");
// eslint-disable-next-line @typescript-eslint/no-var-requires
const path = require("path");

const contentGlobSuffix = "**/*.{svelte,ts}";

const srcContentDir = "./src";

const lgnFrontendContentDir = "../../../node_modules/@lgn/frontend/src";

if (!fs.existsSync(lgnFrontendContentDir)) {
  // eslint-disable-next-line no-console
  console.error(
    `It seems @lgn/frontend src folder is not installed or not accessible here: ${lgnFrontendContentDir}`
  );

  process.exit(1);
}

module.exports = {
  mode: "jit",
  content: [
    "index.html",
    path.join(srcContentDir, contentGlobSuffix),
    path.join(lgnFrontendContentDir, contentGlobSuffix),
  ],
  theme: {
    fontFamily: {
      default: "Source Sans Pro,Arial,sans-serif",
    },
    extend: {
      colors: {
        white: "#eeeeee",
        gray: {
          400: "#666666",
          500: "#555555",
          700: "#333333",
          800: "#222222",
        },
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
