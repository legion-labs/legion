/* eslint-disable @typescript-eslint/no-var-requires */
const tailwindConfig = require("../../tailwind.config.cjs");
const path = require("path");

const { withOpacity } = require("../tailwind.js");

const {
  srcContentDir,
  contentGlobSuffix,
  lgnFrontendContentDir,
} = require("./../tailwind.const.js");

const plugin = require("tailwindcss/plugin");

const themePlugin = plugin(function ({ addComponents }) {
  addComponents({
    ".background": {
      "background-color": "rgb(var(--color-background))",
    },

    ".surface": {
      "background-color": "rgb(var(--color-surface))",
    },

    ".backdrop": {
      "background-color":
        "rgba(var(--color-backdrop), var(--opacity-backdrop))",
    },

    ".notification": {
      "background-color":
        "rgba(var(--color-notification), var(--opacity-notification))",
    },

    ".on-surface": {
      "background-color": "rgb(var(--color-on-surface))",
    },

    ".headline": {
      color: "rgba(var(--color-headline), var(--opacity-headline))",
    },

    ".text": {
      color: "rgba(var(--color-text), var(--opacity-text))",
    },

    ".placeholder": {
      color: "rgba(var(--color-placeholder), var(--opacity-placeholder))",
    },
  });
});

module.exports = {
  ...tailwindConfig,
  content: [
    path.join(srcContentDir, contentGlobSuffix),
    path.join(lgnFrontendContentDir, contentGlobSuffix),
    path.join(lgnFrontendContentDir, "app.html"),
  ],
  theme: {
    extend: {
      borderRadius: {
        DEFAULT: "var(--roundness)",
        xs: "1px",
      },
      colors: {
        primary: withOpacity("--color-primary"),
        accent: withOpacity("--color-accent"),
        // TODO: Remove, used as border-headline in the call graph
        headline: withOpacity("--color-headline", "--opacity-headline"),
        graph: {
          red: withOpacity("--color-graph-red"),
          orange: withOpacity("--color-graph-orange"),
        },
      },
      backgroundColor: {
        default: withOpacity("--background-default"),
      },
      textColor: {
        default: withOpacity("--text-default"),
      },
      minWidth: {
        "thread-item": "var(--thread-item-length)",
      },
    },
  },
  plugins: [themePlugin],
};
