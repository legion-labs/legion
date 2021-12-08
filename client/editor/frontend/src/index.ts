import "./assets/index.css";

import log, { Level } from "@/lib/log";
import App from "./App.svelte";
import { createAwsCognitoTokenCache, finalizeAwsCognitoAuth } from "./lib/auth";

const target = document.querySelector("#root");

if (!target) {
  throw new Error("#root element can't be found");
}

// TODO: Set level from configuration file
const logLevel: Level = "warn";

if (logLevel) {
  log.init();
  log.set(logLevel);
}

// TODO: Make a small router for this
if (window.location.pathname === "/") {
  const code = new URLSearchParams(window.location.search).get("code");

  if (code) {
    const awsCognitoTokenCache = createAwsCognitoTokenCache();

    finalizeAwsCognitoAuth(awsCognitoTokenCache, code).then(() => {
      // Cleanup the Url
      window.history.replaceState(null, "Home", "/");
    });
  }
}

new App({ target });
