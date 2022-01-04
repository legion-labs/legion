import {
  createAwsCognitoTokenCache,
  finalizeAwsCognitoAuth,
  scheduleRefreshClientTokenSet,
  UserInfo,
} from "./lib/auth";
import userInfo from "./stores/userInfo";

/**
 * If a `code` is present in the url then this function will try
 * to authenticate the user, otherwise the global user info store
 * is populated and the user info set returned.
 */
export async function authUser() {
  const awsCognitoTokenCache = createAwsCognitoTokenCache();

  const code =
    window.location.pathname === "/" &&
    new URLSearchParams(window.location.search).get("code");

  if (code) {
    await finalizeAwsCognitoAuth(awsCognitoTokenCache, code);

    window.history.replaceState(null, "Home", "/");
  }

  try {
    const userInfoSet = await userInfo.run();

    // Schedule refresh token only in browsers, not Tauri
    if (!window.__TAURI__) {
      try {
        // Can be freed when needed
        const _timeoutHandle =
          scheduleRefreshClientTokenSet(awsCognitoTokenCache);
      } catch {
        // Empty for now as it should not happen
      }
    }

    return userInfoSet;
  } catch {
    // Force auth
    awsCognitoTokenCache.getAuthorizationCodeInteractive();

    return null;
  }
}

/**
 * Init some of the required dependencies.
 * _Must be called at the beginning of any application that uses this library._
 */
export async function init(options: { auth: true }): Promise<UserInfo>;
export async function init(options: { auth: false }): Promise<null>;
export async function init({ auth }: { auth: boolean }) {
  return auth ? authUser() : null;
}
