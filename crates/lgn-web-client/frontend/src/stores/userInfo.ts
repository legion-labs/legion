import { invoke } from "@tauri-apps/api";
import { UserInfo } from "../lib/auth";
import { createAwsCognito } from "../lib/auth/browser";
import { getCookie } from "../lib/cookie";
import { AsyncStoreOrchestrator } from "./asyncStore";

export async function getAccessToken(): Promise<string | null> {
  return <string | null>(
    (window.__TAURI__
      ? await invoke("plugin:browser|get_access_token")
      : getCookie("access_token"))
  );
}

export async function getUserInfo() {
  const awsCognitoAuthenticator = createAwsCognito();
  const accessToken = await getAccessToken();
  if (!accessToken) {
    throw new Error("Couldn't find access token in cookies");
  }
  return awsCognitoAuthenticator.getUserInfo(accessToken);
}

export default new AsyncStoreOrchestrator<UserInfo>();
