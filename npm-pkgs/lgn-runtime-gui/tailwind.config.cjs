/* eslint-disable @typescript-eslint/no-var-requires */
const tailwindConfig = require("../../tailwind.config.cjs");
const fs = require("fs");
const path = require("path");

const {
  srcContentDir,
  contentGlobSuffix,
  lgnFrontendContentDir,
} = require("./../tailwind.const.js");

if (!fs.existsSync(lgnFrontendContentDir)) {
  // eslint-disable-next-line no-console
  console.error(
    `It seems @lgn/web-client src folder is not installed or not accessible here: ${lgnFrontendContentDir}`
  );

  process.exit(1);
}

/** @type {import("tailwindcss/tailwind-config").TailwindConfig } */
module.exports = {
  ...tailwindConfig,
  content: [
    "index.html",
    path.join(srcContentDir, contentGlobSuffix),
    path.join(lgnFrontendContentDir, contentGlobSuffix),
  ],
};
