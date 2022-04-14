// @ts-check

module.exports = {
  mode: "jit",
  content: ["index.html", "./src/**/*.{svelte,ts}"],
  theme: {
    fontFamily: {
      default: "Inter,Arial,sans-serif",
    },
    extend: {
      backgroundColor: {
        skin: {
          600: "var(--color-background-600)",
          700: "var(--color-background-700)",
          800: "var(--color-background-800)",
          900: "var(--color-background-900)",
        },
      },
      colors: {
        primary: {
          700: "var(--color-primary-700)",
          800: "var(--color-primary-800)",
          900: "var(--color-primary-900)",
        },
        content: {
          38: "var(--color-content-38)",
          60: "var(--color-content-60)",
          87: "var(--color-content-87)",
          100: "var(--color-content-100)",
        },
        white: {
          38: "#FFFFFF61",
          60: "#FFFFFF99",
          87: "#FFFFFFDE",
          100: "#FFFFFFFF",
        },
        black: {
          38: "#00000061",
          60: "#00000099",
          87: "#000000DE",
          100: "#000000FF",
        },
        charcoal: {
          600: "#393939",
          700: "#202020",
          800: "#181818",
          900: "#0F0F0F",
        },
        graph: {
          red: "#EE7B70FF",
          orange: "#F5BB5CFF",
        },
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
