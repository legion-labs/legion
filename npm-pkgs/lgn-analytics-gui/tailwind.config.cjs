// @ts-check

/**
 * @param {string} varName
 * @param {number} [defaultOpacity]
 */
function withOpacity(varName, defaultOpacity) {
  return (/** @type {{opacityValue?: string }} */ { opacityValue }) => {
    const opacity =
      typeof opacityValue !== "undefined" ? opacityValue : defaultOpacity;

    if (typeof opacity !== "undefined") {
      return `rgba(var(${varName}), ${opacity})`;
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
        headline: withOpacity("--color-headline", 87),
        text: withOpacity("--color-text", 60),
        placeholder: withOpacity("--color-placeholder", 38),
        "on-surface": withOpacity("--color-on-surface"),
        notification: withOpacity("--color-notification"),

        // TODO: Revamp
        graph: {
          red: "#EE7B70FF",
          orange: "#F5BB5CFF",
        },

        // TODO: Remove all
        content: {
          38: "var(--color-content-38)",
          60: "var(--color-content-60)",
          87: "var(--color-content-87)",
          100: "var(--color-content-100)",
        },
        white: {
          87: "#FFFFFFDE",
          100: "#FFFFFFFF",
        },
        black: {
          87: "#000000DE",
        },
        charcoal: {
          600: "#393939",
        },
      },
      backgroundColor: {
        background: withOpacity("--color-background"),
        surface: withOpacity("--color-surface"),
        backdrop: withOpacity("--color-backdrop", 87),
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
