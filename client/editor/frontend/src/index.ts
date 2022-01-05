import "./assets/index.css";

import log, { Level } from "@lgn/frontend/src/lib/log";
import { init as initLgnFrontend } from "@lgn/frontend";
import App from "@/App.svelte";
import "@/workers/editorWorker";

/**
 * Runs the application.
 */
async function run() {
  const target = document.querySelector("#root");

  if (!target) {
    log.error("#root element can't be found");

    return;
  }

  const logLevel: Level = "warn";

  if (logLevel) {
    log.init();
    log.set(logLevel);
  }

  await initLgnFrontend({ auth: true, forceAuth: false });

  try {
    new App({ target });
  } catch (error) {
    log.error(error);

    return;
  }
}

run();
