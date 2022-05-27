/** @type {import("tailwindcss/tailwind-config").TailwindConfig } */
module.exports = {
  theme: {
    fontFamily: {
      default: "Inter,Arial,sans-serif",
    },
    extend: {
      backgroundColor: {
        surface: {
          500: "var(--color-background-500)",
          600: "var(--color-background-600)",
          700: "var(--color-background-700)",
          800: "var(--color-background-800)",
          900: "var(--color-background-900)",
          max: "var(--color-background-max)",
        },
        list: {
          "item-even": "var(--list-item-bg-even)",
          "item-odd": "var(--list-item-bg-odd)",
          header: "var(--list-item-bg-header)",
        },
        vector: {
          x: "var(--color-vector-x)",
          y: "var(--color-vector-y)",
          z: "var(--color-vector-z)",
          w: "var(--color-vector-w)",
        },
      },
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
};
