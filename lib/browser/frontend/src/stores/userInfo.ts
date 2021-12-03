import { invoke } from "@tauri-apps/api";

import { getAccessToken, getUserInfo, UserInfo } from "@lgn/browser-auth";
import asyncData from "./asyncData";

export default asyncData<UserInfo>(async () => {
  const accessToken = window.__TAURI__
    ? await invoke("plugin:browser|get_access_token")
    : getAccessToken();

  if (!accessToken) {
    throw new Error("Couldn't find access token in cookies");
  }

  return getUserInfo(accessToken);
});
