import { invoke } from "@tauri-apps/api";

import userInfoStore, {
  getUserInfo as _getUserInfo,
} from "../../stores/userInfo";
import log from "../log";

/**
 * Start authentication on Tauri.
 */
export async function startUserAuth() {
  try {
    await userInfoStore.run(async () => {
      const userInfo = await invoke("plugin:browser|authenticate");

      log.debug("auth", userInfo);

      return userInfo;
    });
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
export async function getUserInfo({ forceAuth }: { forceAuth: boolean }) {
  try {
    await userInfoStore.run(_getUserInfo);
  } catch {
    if (forceAuth) {
      await startUserAuth();
    }
  }
}
