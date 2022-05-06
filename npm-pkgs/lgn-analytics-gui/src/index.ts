import { navigate } from "svelte-navigator";

import { AppComponent, run } from "@lgn/web-client";

import App from "./App.svelte";
import "./assets/index.css";

const redirectUri = document.location.origin + "/";

run({
  appComponent: App as typeof AppComponent,
  auth: {
    issuerUrl:
      "https://cognito-idp.ca-central-1.amazonaws.com/ca-central-1_SkZKDimWz",
    redirectUri,
    clientId: "2kp01gr54dfc7qp1325hibcro3",
    login: {
      cookies: {
        accessToken: "analytics_access_token_v2",
        refreshToken: "analytics_refresh_token_v2",
      },
      scopes: ["email", "openid", "profile"],
    },
    redirectFunction(url) {
      return navigate(url.toString(), { replace: true });
    },
  },
  rootQuerySelector: "#root",
  logLevel: "debug",
})
  // eslint-disable-next-line no-console
  .catch((error) => console.error("Application couldn't start", error));
