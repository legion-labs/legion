// @ts-check

/* eslint-disable @typescript-eslint/no-var-requires */

const { contextBridge, ipcRenderer } = require("electron");
const { getAll } = require("@lgn/config");
const auth = require("@lgn/auth");

const config = getAll({
  issuerUrl: "online.authentication.issuer_url",
  clientId: "online.authentication.client_id",
});

const redirectUri = "http://localhost:5000";

contextBridge.exposeInMainWorld("isElectron", true);

contextBridge.exposeInMainWorld("electron", {
  toggleMaximizeMainWindow: () => {
    ipcRenderer.send("main-window-toggle-maximize");
  },
  minimizeMainWindow: () => {
    ipcRenderer.send("main-window-minimize");
  },
  closeMainWindow: () => {
    ipcRenderer.send("main-window-close");
  },
  auth: {
    initOAuthClient() {
      return auth.initOAuthClient(
        "legion-editor",
        // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
        config["issuerUrl"],
        // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
        config["clientId"],
        redirectUri
      );
    },
    authenticate: auth.authenticate,
    getAccessToken: auth.getAccessToken,
  },
});
