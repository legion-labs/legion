import "./assets/index.css";
import { defaultAuthUserConfig, run } from "@lgn/frontend";
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
  onPreInit: null,
})
  // eslint-disable-next-line no-console
  .catch((error) => console.error("Application couldn't start", error));
