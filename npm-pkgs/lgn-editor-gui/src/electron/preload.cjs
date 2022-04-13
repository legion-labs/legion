// @ts-check

/* eslint-disable @typescript-eslint/no-var-requires */

const { contextBridge, ipcRenderer } = require("electron");

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
});
