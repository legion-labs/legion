import { invoke } from "@tauri-apps/api";
import {
  getAccessToken,
  getUserInfo as authGetUserInfo,
  UserInfo,
} from "@lgn/browser-auth";
import asyncStore from "./asyncStore";

export async function getUserInfo() {
  const accessToken = window.__TAURI__
    ? await invoke("plugin:browser|get_access_token")
    : getAccessToken();

  if (!accessToken) {
    throw new Error("Couldn't find access token in cookies");
  }

  return authGetUserInfo(accessToken);
}

export default asyncStore<UserInfo>();
