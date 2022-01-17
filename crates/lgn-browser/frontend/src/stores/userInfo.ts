import { invoke } from "@tauri-apps/api";
import { UserInfo } from "../lib/auth";
import { createAwsCognito } from "../lib/auth/browser";
import { getCookie } from "../lib/cookie";
import asyncStore from "./asyncStore";

export async function getUserInfo() {
  const awsCognitoAuthenticator = createAwsCognito();

  const accessToken = window.__TAURI__
    ? await invoke("plugin:browser|get_access_token")
    : getCookie("access_token");

  if (!accessToken) {
    throw new Error("Couldn't find access token in cookies");
  }

  return awsCognitoAuthenticator.getUserInfo(accessToken);
}

export default asyncStore<UserInfo>();
