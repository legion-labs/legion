/* eslint-disable @typescript-eslint/no-var-requires */

const path = require("path");

const contentGlobSuffix = "**/*.{svelte,ts}";

const srcContentDir = "./src";

const lgnFrontendContentDir = "./node_modules/@lgn/web-client/src";

// It's really painful that the type denition doesn't support commonjs
/** @type {any} */
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, no-undef
const plugin_ = require("tailwindcss/plugin");
/** @type {import("tailwindcss/plugin").TailwindPluginCreator} */
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
const plugin = plugin_;

/**
 * @param {string} varName
 * @param {string} [defaultOpacityVariable]
 */
function withOpacity(varName, defaultOpacityVariable) {
  return (/** @type {{opacityVariable?: string }} */ { opacityVariable }) => {
    const opacity =
      typeof defaultOpacityVariable !== "undefined"
        ? defaultOpacityVariable
        : opacityVariable;

    if (typeof opacity !== "undefined") {
      return `rgba(var(${varName}), var(${opacity}))`;
    }

    return `rgb(var(${varName}))`;
  };
}

// TODO: Extract to @lgn/web-client and reuse in editor
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call
const themePlugin = plugin(function ({ addComponents }) {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call
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

// eslint-disable-next-line no-undef
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
      borderRadius: {
        DEFAULT: "var(--roundness)",
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
  variants: {
    extend: {},
  },
  plugins: [themePlugin],
};
