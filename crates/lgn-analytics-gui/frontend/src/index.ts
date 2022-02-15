import "./assets/index.css";
import { run } from "@lgn/web-client";
import App from "./App.svelte";

run({
  appComponent: App,
  auth: {
    forceAuth: true,
    redirectionTitle: "Home",
    issuerUrl:
      "https://cognito-idp.ca-central-1.amazonaws.com/ca-central-1_SkZKDimWz",
    redirectUri: "http://localhost:3000/",
    clientId: "5m58nrjfv6kr144prif9jk62di",
    login: {
      cookies: {
        accessToken: "analytics_access_token",
        refreshToken: "analytics_refresh_token",
      },
      extraParams: {
        identity_provider: "Azure",
      },
      scopes: [
        "aws.cognito.signin.user.admin",
        "email",
        "https://legionlabs.com/editor/allocate",
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
