import "./assets/index.css";

import { AppComponent, run } from "@lgn/web-client";
import App from "@/App.svelte";
import "@/workers/editorWorker";
import contextMenu from "@/stores/contextMenu";
import * as contextMenuEntries from "@/data/contextMenu";
import viewportOrchestrator from "@/stores/viewport";
//import initWasmLogger, { debug } from "@lgn/simple-wasm-logger";

const redirectUri = document.location.origin + "/";

run({
  appComponent: App as typeof AppComponent,
  auth: {
    redirectionTitle: "Home",
    issuerUrl:
      "https://cognito-idp.ca-central-1.amazonaws.com/ca-central-1_SkZKDimWz",
    redirectUri,
    clientId: "5m58nrjfv6kr144prif9jk62di",
    login: {
      cookies: {
        accessToken: "editor_access_token",
        refreshToken: "editor_refresh_token",
      },
      extraParams: {
        // eslint-disable-next-line camelcase
        identity_provider: "Azure",
      },
      scopes: [
        "aws.cognito.signin.user.admin",
        "email",
        "https://legionlabs.com/editor/allocate",
        "openid",
        "profile",
      ],
    },
  },
  rootQuerySelector: "#root",
  logLevel: "warn",
  async onPreInit() {
    //await initWasmLogger();
    //debug("Hello from the Legion editor");

    contextMenu.register("resource", contextMenuEntries.resourceEntries);
    contextMenu.register(
      "resourcePanel",
      contextMenuEntries.resourcePanelEntries
    );

    const editorViewportKey = Symbol();

    viewportOrchestrator.addAllViewport(
      [editorViewportKey, { type: "video", name: "editor" }],
      [Symbol(), { type: "video", name: "runtime" }]
    );

    viewportOrchestrator.activate(editorViewportKey);
  },
})
  // eslint-disable-next-line no-console
  .catch((error) => console.error("Application couldn't start", error));
