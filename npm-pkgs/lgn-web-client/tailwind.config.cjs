module.exports = {
  mode: "jit",
  content: ["index.html", "./src/**/*.{svelte,ts}"],
  theme: {
    fontFamily: {
      default: "Inter,Arial,sans-serif",
    },
    extend: {
      colors: {
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
