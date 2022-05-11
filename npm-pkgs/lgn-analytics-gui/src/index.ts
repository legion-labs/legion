import { navigate } from "svelte-navigator";

import { AppComponent, run } from "@lgn/web-client";
import type { Level } from "@lgn/web-client/src/lib/log";
import {
  ConsoleTransport,
  NotificationsTransport,
} from "@lgn/web-client/src/lib/log/transports";
import { createNotificationsStore } from "@lgn/web-client/src/stores/notifications";

import App from "./App.svelte";
import "./assets/index.css";

const redirectUri = document.location.origin + "/";

const notifications = createNotificationsStore<Fluent>();

run({
  appComponent: App as typeof AppComponent,
  extraProps: { notifications },
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
  log: {
    transports: [
      new ConsoleTransport({
        level: import.meta.env.VITE_LEGION_ANALYTICS_CONSOLE_LOG_LEVEL as Level,
      }),
      new NotificationsTransport<Fluent>({
        notificationsStore: notifications,
        level: import.meta.env
          .VITE_LEGION_ANALYTICS_NOTIFICATION_LOG_LEVEL as Level,
      }),
    ],
  },
})
  // eslint-disable-next-line no-console
  .catch((error) => console.error("Application couldn't start", error));
