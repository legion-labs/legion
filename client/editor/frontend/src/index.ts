import "./assets/index.css";

import { defaultAuthUserConfig, run } from "@lgn/frontend";
import App from "@/App.svelte";
import "@/workers/editorWorker";
import initWasmLogger, { debug } from "@lgn/simple-wasm-logger";

run({
  appComponent: App,
  auth: defaultAuthUserConfig(),
  rootQuerySelector: "#root",
  logLevel: "warn",
  async onPreInit() {
    await initWasmLogger();
    debug("Hello from the Legion editor");
  },
})
  // eslint-disable-next-line no-console
  .catch((error) => console.error("Application couldn't start", error));
