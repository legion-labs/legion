import { l10nOrchestratorContextKey as originalL10nOrchestratorContextKey } from "@lgn/web-client/src/constants";

/** Key to the theme context */
export const themeContextKey = Symbol.for("theme-context-key");
/** Key to the thread item CSS width in pixel context */
export const threadItemLengthContextKey = Symbol.for(
  "thread-item-length-context-key"
);
/** Key to the l10n store set context */
export const l10nOrchestratorContextKey = originalL10nOrchestratorContextKey;
/** Key to the http client context */
export const httpClientContextKey = Symbol.for("http-client-context-key");
/** Key to the notifications context */
export const notificationsContextKey = Symbol.for("notifications-context-key");
/** Key to the notifications context */
export const debugContextKey = Symbol.for("debug-context-key");

export const themeStorageKey = "theme";
export const localeStorageKey = "locale";

/** Abitrary thread item lenght used if the proper one cannot be computed, should never be used */
export const threadItemLengthFallback = 170;
