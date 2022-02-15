import "./assets/index.css";
import { run } from "@lgn/web-client";
import App from "./App.svelte";

const redirectUri = document.location.origin + "/"

run({
  appComponent: App,
  auth: {
    forceAuth: true,
    redirectionTitle: "Home",
    issuerUrl:
      "https://cognito-idp.ca-central-1.amazonaws.com/ca-central-1_SkZKDimWz",
    redirectUri: redirectUri,
    clientId: "2kp01gr54dfc7qp1325hibcro3",
    login: {
      cookies: {
        accessToken: "analytics_access_token_v2",
        refreshToken: "analytics_refresh_token_v2",
      },
      scopes: [
        "email",
        "openid",
        "profile",
      ],
    },
  },
  rootQuerySelector: "#root",
  logLevel: "debug",
  onPreInit: undefined,
})
  // eslint-disable-next-line no-console
  .catch((error) => console.error("Application couldn't start", error));
