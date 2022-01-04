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

  const userInfo = await initLgnFrontend({ auth: true });

  log.debug(
    "user",
    userInfo ? log.json`User is authed: ${userInfo}` : "User is not authed"
  );

  try {
    new App({ target });
  } catch (error) {
    log.error(error);

    return;
  }
}

run();
