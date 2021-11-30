import "./assets/index.css";

import log, { Level } from "@/lib/log";
import App from "./App.svelte";

// TODO: Set level from configuration file
const logLevel: Level = "debug";

if (logLevel) {
  log.init();
  log.set(logLevel);
}

const target = document.querySelector("#root");

if (!target) {
  throw new Error("#root element can't be found");
}

new App({ target });
