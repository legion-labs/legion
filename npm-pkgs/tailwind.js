/** @type {import("./tailwind.types").TailwindTransform } */
module.exports = {
  withOpacity: (varName, defaultOpacityVariable) => {
    return (p) => {
      const opacity =
        typeof defaultOpacityVariable !== "undefined"
          ? defaultOpacityVariable
          : p.opacityVariable;

      if (typeof opacity !== "undefined") {
        return `rgba(var(${varName}), var(${opacity}))`;
      }

      return `rgb(var(${varName}))`;
    };
  },
};
