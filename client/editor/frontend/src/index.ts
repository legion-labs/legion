import "./assets/index.css";

import log, { Level } from "@/lib/log";
import userInfo from "@/stores/userInfo";
import { createAwsCognitoTokenCache, finalizeAwsCognitoAuth } from "@/lib/auth";
import App from "@/App.svelte";

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

const code =
  window.location.pathname === "/" &&
  new URLSearchParams(window.location.search).get("code");

if (code) {
  const awsCognitoTokenCache = createAwsCognitoTokenCache();

  finalizeAwsCognitoAuth(awsCognitoTokenCache, code)
    .then((newUserInfo) => {
      if (newUserInfo) {
        userInfo.data.set(newUserInfo);
      }
    })
    .then(() => {
      // Cleanup the Url
      window.history.replaceState(null, "Home", "/");
    });
}

// Fetch user info before running the application
userInfo
  .run()
  .then((_userInfo) => {
    log.debug("User is authed");
  })
  .catch(() => {
    log.debug("User is not authed");
  })
  .finally(() => {
    new App({ target });
  });
