import { invoke } from "@tauri-apps/api";

import { UserInfo } from ".";
import { AsyncStore } from "../../stores/asyncStore";
import log from "../log";

/**
 * Start authentication on Tauri.
 */
export async function startUserAuth(asyncStore: AsyncStore<UserInfo>) {
  try {
    const userInfo = await invoke("plugin:browser|authenticate");

    log.debug("auth", userInfo);

    asyncStore.data.set(userInfo);
  } catch {
    // Nothing we can do about this but warn the user
    log.error("Couldn't authenticate the user");
  }
}

/**
 * Fetch user info in Tauri
 *
 * If the `forceAuth` option is `true` the unauthenticated users
 * will be redirected to Cognito.
 */
export async function getUserInfo(
  asyncStore: AsyncStore<UserInfo>,
  { forceAuth }: { forceAuth: boolean }
) {
  try {
    return await asyncStore.run();
  } catch {
    if (forceAuth) {
      startUserAuth(asyncStore);
    }

    return null;
  }
}
