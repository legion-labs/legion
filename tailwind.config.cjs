/** @type {import("tailwindcss/tailwind-config").TailwindConfig } */
module.exports = {
  mode: "jit",
  content: ["npm-pkgs/**/src/index.html", "npm-pkgs/**/src/**/*.{svelte,ts}"],
  presets: [require("./tailwind.preset.cjs")],
};
