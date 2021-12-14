// eslint-disable-next-line @typescript-eslint/no-var-requires
const preprocess = require("svelte-preprocess");

module.exports = {
  preprocess: preprocess({
    postcss: true,
    typescript: true,
  }),
};
