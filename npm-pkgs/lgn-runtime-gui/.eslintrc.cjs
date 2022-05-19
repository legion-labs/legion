// @ts-check

module.exports = {
  env: {
    browser: true,
    es2021: true,
    node: true,
  },
  parser: "@typescript-eslint/parser",
  parserOptions: {
    tsconfigRootDir: __dirname,
    project: "tsconfig.json",
    extraFileExtensions: [".cjs", ".svelte"],
  },
  plugins: ["svelte3", "@typescript-eslint"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:@typescript-eslint/recommended-requiring-type-checking",
  ],
  overrides: [
    {
      files: ["*.svelte"],
      processor: "svelte3/svelte3",
      rules: {
        // TODO: The following rules don't work well with Svelte, try to fix them
        "@typescript-eslint/no-unsafe-member-access": "off",
        "@typescript-eslint/no-unsafe-call": "off",
        "@typescript-eslint/restrict-plus-operands": "off",
        "@typescript-eslint/restrict-template-expressions": "off",
        "@typescript-eslint/no-unsafe-assignment": "off",
        "@typescript-eslint/no-unsafe-argument": "off",
      },
    },
  ],
  settings: {
    "svelte3/typescript": true,
    "svelte3/ignore-styles": () => true,
  },
  rules: {
    camelcase: "error",
    "no-console": process.env.NODE_ENV === "production" ? "error" : "off",
    "no-debugger": process.env.NODE_ENV === "production" ? "error" : "off",
    "@typescript-eslint/no-unused-vars": "off",
    "no-var": "error",
    "padding-line-between-statements": [
      "error",
      { blankLine: "always", prev: "*", next: "return" },
      { blankLine: "always", prev: "*", next: "block-like" },
      { blankLine: "always", prev: ["const", "let", "var"], next: "*" },
      {
        blankLine: "any",
        prev: ["const", "let", "var"],
        next: ["const", "let", "var"],
      },
    ],
    "prefer-const": [
      "error",
      {
        destructuring: "any",
        ignoreReadBeforeAssign: false,
      },
    ],
    eqeqeq: "error",
    "@typescript-eslint/strict-boolean-expressions": "error",
  },
};
