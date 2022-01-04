import { invoke } from "@tauri-apps/api";

import { createAwsCognito } from "../lib/auth";
import { getCookie } from "../lib/cookie";
import asyncData from "./asyncData";

export default asyncData(async () => {
  const awsCognitoAuthenticator = createAwsCognito();

  const accessToken = window.__TAURI__
    ? await invoke("plugin:browser|get_access_token")
    : getCookie("access_token");

  if (!accessToken) {
    throw new Error("Couldn't find access token in cookies");
  }

  return awsCognitoAuthenticator.getUserInfo(accessToken);
});
