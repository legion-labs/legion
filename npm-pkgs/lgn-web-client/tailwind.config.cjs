module.exports = {
  mode: "jit",
  content: ["index.html", "./src/**/*.{svelte,ts}"],
  theme: {
    fontFamily: {
      default: "Inter,Arial,sans-serif",
    },
    extend: {
      colors: {
        content: {
          500: "var(--color-background-500)",
          600: "var(--color-background-600)",
          700: "var(--color-background-700)",
          800: "var(--color-background-800)",
          900: "var(--color-background-900)",
          max: "var(--color-background-max)",
        },
        item: {
          max: "var(--color-item-max)",
          high: "var(--color-item-high)",
          mid: "var(--color-item-mid)",
          low: "var(--color-item-low)",
          min: "var(--color-item-min)",
        },
        white: "#eeeeee",
        black: "#181818",
        gray: {
          400: "#666666",
          500: "#555555",
          700: "#333333",
          800: "#222222",
        },
        orange: {
          700: "#fc4d0f",
        },
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
