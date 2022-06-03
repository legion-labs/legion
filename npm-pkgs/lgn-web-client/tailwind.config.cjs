// eslint-disable-next-line @typescript-eslint/no-var-requires
const tailwindConfig = require("../../tailwind.config.cjs");

module.exports = {
  ...tailwindConfig,
  content: ["index.html", "./src/**/*.{svelte,ts}"],
};
