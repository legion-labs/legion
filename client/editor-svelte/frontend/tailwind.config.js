module.exports = {
  mode: "jit",
  purge: ["index.html", "./src/**/*.{svelte,ts}"],
  darkMode: false,
  theme: {
    fontFamily: {
      default: "Source Sans Pro,Arial,sans-serif",
    },
    extend: {
      colors: {
        white: "#eeeeee",
        gray: {
          400: "#666666",
          500: "#555555",
          700: "#333333",
          800: "#222222",
        },
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
