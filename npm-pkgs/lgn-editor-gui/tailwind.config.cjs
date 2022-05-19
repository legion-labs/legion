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

/** @type {import("tailwindcss/tailwind-config").TailwindConfig } */
module.exports = {
  ...require("../../tailwind.config.cjs"),
  content: [
    path.join(srcContentDir, contentGlobSuffix),
    path.join(lgnFrontendContentDir, contentGlobSuffix),
    path.join(lgnFrontendContentDir, "app.html"),
  ],
};
