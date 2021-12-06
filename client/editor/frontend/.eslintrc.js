module.exports = {
  env: {
    browser: true,
    es2021: true,
  },
  parser: "@typescript-eslint/parser",
  plugins: ["svelte3"],
  extends: ["plugin:@typescript-eslint/recommended"],
  overrides: [
    {
      files: ["*.svelte"],
      processor: "svelte3/svelte3",
    },
  ],
  settings: {
    "svelte3/typescript": true,
    "svelte3/ignore-styles": () => true,
  },
  rules: {
    "no-console": "error",
    "@typescript-eslint/no-unused-vars": "off",
  },
};
