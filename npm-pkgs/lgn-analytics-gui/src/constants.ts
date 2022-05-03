/** Key to the theme context */
export const themeContextKey = Symbol.for("theme-context-key");
/** Key to the thread item CSS width in pixel context */
export const threadItemLengthContextKey = Symbol.for(
  "thread-item-length-context-key"
);

export const themeStorageKey = "theme";
export const localeStorageKey = "locale";

/** Abitrary thread item lenght used if the proper one cannot be computed, should never be used */
export const threadItemLengthFallback = 170;
