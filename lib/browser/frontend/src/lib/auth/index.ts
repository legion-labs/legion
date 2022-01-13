import { startUserAuth as browserStartUserAuth } from "./browser";
import { startUserAuth as tauriStartUserAuth } from "./tauri";
import { UserInfo as BrowserAuthUserInfo } from "@lgn/browser-auth";

// @lgn/browser-auth is used as the source of truth but it's basically
// the same code as in lgn-online copy pasted.
export type UserInfo = BrowserAuthUserInfo;

/**
 * Start user authentication on Tauri or Browser.
 *
 * You can use the specialized `tauriStartUserAuth` and `browserStartUserAuth`
 * if needed, but be aware authentication might break if not used properly.
 */
export function startUserAuth() {
  if (window.__TAURI__) {
    return tauriStartUserAuth();
  } else {
    return browserStartUserAuth();
  }
}
