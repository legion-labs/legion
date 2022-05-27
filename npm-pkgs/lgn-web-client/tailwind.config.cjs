// eslint-disable-next-line @typescript-eslint/no-var-requires
const defaultConfig = require("../../tailwind.config.cjs");

module.exports = {
  ...defaultConfig,
  content: ["index.html", "./src/**/*.{svelte,ts}"],
};
