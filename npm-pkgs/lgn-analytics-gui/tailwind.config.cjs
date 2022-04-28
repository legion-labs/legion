// @ts-check

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

// eslint-disable-next-line no-undef
module.exports = {
  mode: "jit",
  content: ["index.html", "./src/**/*.{svelte,ts}"],
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
        headline: withOpacity("--color-headline", "--opacity-headline"),
        text: withOpacity("--color-text", "--opacity-text"),
        placeholder: withOpacity(
          "--color-placeholder",
          "--opacity-placeholder"
        ),
        "on-surface": withOpacity("--color-on-surface"),
        notification: withOpacity("--color-notification"),
        graph: {
          red: withOpacity("--color-graph-red"),
          orange: withOpacity("--color-graph-orange"),
        },
      },
      backgroundColor: {
        default: withOpacity("--background-default"),
        background: withOpacity("--color-background"),
        surface: withOpacity("--color-surface"),
        backdrop: withOpacity("--color-backdrop", "--opacity-backdrop"),
      },
      textColor: {
        default: withOpacity("--text-default"),
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
