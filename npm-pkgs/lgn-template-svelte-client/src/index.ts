import { defaultAuthUserConfig, run } from "@lgn/frontend/src";
import App from "./App.svelte";

run({
  appComponent: App,
  auth: defaultAuthUserConfig(),
  rootQuerySelector: "#root",
  logLevel: "warn",
})
  // eslint-disable-next-line no-console
  .catch((error) => console.error("Application couldn't start", error));
