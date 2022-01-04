import { AsyncStore } from "../../stores/asyncStore";
import { startUserAuth as browserStartUserAuth } from "./browser";
import { startUserAuth as tauriStartUserAuth } from "./tauri";

export type UserInfo = {
  sub: string;
  name?: string;
  given_name?: string;
  family_name?: string;
  middle_name?: string;
  nickname?: string;
  preferred_username?: string;
  profile?: string;
  picture?: string;
  website?: string;
  email?: string;
  email_verified?: "true" | "false";
  gender?: string;
  birthdate?: string;
  zoneinfo?: string;
  locale?: string;
  phone_number?: string;
  phone_number_verified?: "true" | "false";
  updated_at?: string;
  // Azure-specific fields.
  //
  // This is a merely a convention, but we need one.
  //
  // These fields contains the Azure-specific information about the user, which allow us to query
  // the Azure API for extended user information (like the user's photo).
  "custom:azure_oid"?: string;
  "custom:azure_tid"?: string;
};

/**
 * Start user authentication on Tauri or Browser.
 *
 * You can use the specialized `tauriStartUserAuth` and `browserStartUserAuth`
 * if needed, but be aware authentication might break if not used properly.
 */
export function startUserAuth(asyncStore: AsyncStore<UserInfo>) {
  if (window.__TAURI__) {
    return tauriStartUserAuth(asyncStore);
  } else {
    return browserStartUserAuth();
  }
}
