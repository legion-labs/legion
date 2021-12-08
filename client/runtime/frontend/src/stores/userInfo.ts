import { createAwsCognito } from "@/lib/auth";
import { getCookie } from "@/lib/cookie";
import { invoke } from "@tauri-apps/api";
import asyncData from "./asyncData";

export default asyncData(async () => {
  const awsCognitoAuthenticator = createAwsCognito();

  const accessToken = window.__TAURI__
    ? await invoke("get_access_token")
    : getCookie("access_token");

  if (!accessToken) {
    return null;
  }

  return awsCognitoAuthenticator.getUserInfo(accessToken);
});
