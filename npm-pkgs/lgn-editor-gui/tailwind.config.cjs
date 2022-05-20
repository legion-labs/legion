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
    path.join(srcContentDir, contentGlobSuffix),
    path.join(lgnFrontendContentDir, contentGlobSuffix),
    path.join(lgnFrontendContentDir, "app.html"),
  ],
  theme: {
    fontFamily: {
      default: "Inter,Arial,sans-serif",
    },
    extend: {
      backgroundColor: {
        surface: {
          700: "var(--color-background-700)",
          800: "var(--color-background-800)",
          900: "var(--color-background-900)",
          max: "var(--color-background-max)",
        },
        list: {
          "item-even": "var(--list-item-bg-even)",
          "item-odd": "var(--list-item-bg-odd)",
          header: "var(--list-item-bg-header)",
        },
        vector: {
          x: "var(--color-vector-x)",
          y: "var(--color-vector-y)",
          z: "var(--color-vector-z)",
          w: "var(--color-vector-w)",
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
