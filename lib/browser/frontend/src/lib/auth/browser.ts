import {
  finalizeAwsCognitoAuth,
  scheduleRefreshClientTokenSet,
  getAuthorizationCodeInteractive,
} from "@lgn/browser-auth";
import userInfo, { getUserInfo } from "../../stores/userInfo";

/**
 * Start authentication in Browser.
 */
export async function startUserAuth() {
  return getAuthorizationCodeInteractive();
}

/**
 * If a `code` is present in the url then this function will try
 * to authenticate the user, otherwise the global user info store
 * is populated and the user info set returned.
 *
 * If the `forceAuth` option is `true` the unauthenticated users
 * will be redirected to Cognito.
 */
export async function userAuth({ forceAuth }: { forceAuth: boolean }) {
  const code =
    window.location.pathname === "/" &&
    new URLSearchParams(window.location.search).get("code");

  if (code) {
    await finalizeAwsCognitoAuth(code);

    window.history.replaceState(null, "Home", "/");
  }

  try {
    const userInfoSet = await getUserInfo();

    userInfo.data.set(userInfoSet);

    // TODO: The returned timeout id can and should be freed.
    // Schedule refresh token.
    scheduleRefreshClientTokenSet();

    return userInfoSet;
  } catch {
    if (forceAuth) {
      startUserAuth();
    }

    return null;
  }
}
