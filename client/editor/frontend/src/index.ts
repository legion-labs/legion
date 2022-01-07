import "./assets/index.css";

import { defaultAuthUserConfig, run } from "@lgn/frontend";
import App from "@/App.svelte";
import "@/workers/editorWorker";

run({
  appComponent: App,
  auth: defaultAuthUserConfig(),
  rootQuerySelector: "#root",
  logLevel: "warn",
})
  // eslint-disable-next-line no-console
  .catch((error) => console.error("Application couldn't start", error));
