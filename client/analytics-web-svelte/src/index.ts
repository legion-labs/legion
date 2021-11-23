import "./assets/index.css";

import App from "./App.svelte";

const target = document.querySelector("#root");

if (!target) {
  throw new Error("#root element can't be found");
}

new App({ target });
