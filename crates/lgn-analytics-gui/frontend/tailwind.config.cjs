// @ts-check

module.exports = {
  mode: "jit",
  content: ["index.html", "./src/**/*.{svelte,ts}"],
  theme: {
    fontFamily: {
      default: "Source Sans Pro,Arial,sans-serif",
    },
    extend: {
      colors: {
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
        gray: {
          300: "#E1E3E7",
          400: "#404040",
          700: "#333333",
          800: "#222222",
        },
        charcoal: {
          600: "#393939",
          700: "#202020",
          800: "#181818",
          900: "#0F0F0F",
        },
        orange: {
          700: "#FC4D0F",
          800: "#E7440A",
          900: "#D83B03",
        },
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
