import initBrowserAuth, { finalizeAwsCognitoAuth } from "@lgn/browser-auth";
import userInfo from "./stores/userInfo";

/**
 * If a `code` is present in the url then this function will try
 * to authenticate the user, otherwise the global user info store
 * is populated and the user info set returned.
 */
export async function authUser() {
  const code =
    window.location.pathname === "/" &&
    new URLSearchParams(window.location.search).get("code");

  if (code) {
    await finalizeAwsCognitoAuth(code);

    window.history.replaceState(null, "Home", "/");
  }

  // TODO: Add a timeout using the expires in value that will "force auth" the user before the tokens expire

  return userInfo.run().catch(() => null);
}

/**
 * Init some of the required dependencies.
 * _Must be called at the beginning of any application that uses this library._
 */
export function init() {
  return Promise.all([initBrowserAuth()]);
}
