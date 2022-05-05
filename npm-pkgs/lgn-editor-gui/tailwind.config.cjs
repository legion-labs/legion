/* eslint-disable @typescript-eslint/no-var-requires */

const fs = require("fs");
const path = require("path");

const contentGlobSuffix = "**/*.{svelte,ts}";

const srcContentDir = "./src";

const lgnFrontendContentDir = "./node_modules/@lgn/web-client/src";

if (!fs.existsSync(lgnFrontendContentDir)) {
  // eslint-disable-next-line no-console
  console.error(
    `It seems @lgn/web-client src folder is not installed or not accessible here: ${lgnFrontendContentDir}`
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
      backgroundColor: {
        surface: {
          700: "var(--color-background-700)",
          800: "var(--color-background-800)",
          900: "var(--color-background-900)",
        },
      },
      colors: {
        white: "#eeeeee",
        black: "#181818",
        gray: {
          400: "#666666",
          500: "#555555",
          700: "#333333",
          800: "#222222",
        },
        orange: {
          700: "#fc4d0f",
        },
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
