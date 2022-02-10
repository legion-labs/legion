import "./assets/index.css";
import { defaultAuthUserConfig, run } from "@lgn/web-client";
import App from "./App.svelte";

run({
  appComponent: App,
  auth: {
    forceAuth: true,
    redirectTo: "/",
    redirectedTo: "/",
    redirectionTitle: "Home",
  },
  rootQuerySelector: "#root",
  logLevel: "debug",
  onPreInit: undefined,
})
  // eslint-disable-next-line no-console
  .catch((error) => console.error("Application couldn't start", error));
