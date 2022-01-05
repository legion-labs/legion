import { getUserInfo as tauriGetUserInfo } from "./lib/auth/tauri";
import { userAuth as browserUserAuth } from "./lib/auth/browser";
import log from "./lib/log";
import userInfo from "./stores/userInfo";
import { UserInfo } from "./lib/auth";

/**
 * Init some of the required dependencies.
 * _Must be called at the beginning of any application that uses this library._
 *
 * If the `forceAuth` option is `true` the unauthenticated users
 * will be redirected to Cognito.
 */
export async function init({
  auth,
  forceAuth,
}: {
  auth: boolean;
  forceAuth: boolean;
}) {
  let userInfoSet: UserInfo | null = null;

  if (auth) {
    if (window.__TAURI__) {
      userInfoSet = await tauriGetUserInfo(userInfo, { forceAuth });
    } else {
      userInfoSet = await browserUserAuth(userInfo, { forceAuth });
    }
  }

  log.debug(
    "user",
    userInfoSet
      ? log.json`User is authed: ${userInfoSet}`
      : "User is not authed"
  );
}
