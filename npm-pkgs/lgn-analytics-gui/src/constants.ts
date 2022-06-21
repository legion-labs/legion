import { getEnv } from "@lgn/web-client/src/lib/env";

export const themeStorageKey = "theme";
export const localeStorageKey = "locale";

/** Abitrary thread item lenght used if the proper one cannot be computed, should never be used */
export const threadItemLengthFallback = 170;

const allowedApp = "analytics";

const allowedDomain = "legionengine.com";

export const env = getEnv({ allowedApp, allowedDomain });

export const accessTokenCookieName =
  "analytics_access_token_v2" + (env ? `_${env}` : "");

export const refreshTokenCookieName =
  "analytics_refresh_token_v2" + (env ? `_${env}` : "");
