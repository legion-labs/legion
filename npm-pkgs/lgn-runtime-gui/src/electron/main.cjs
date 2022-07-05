// @ts-check

/* eslint-disable @typescript-eslint/no-var-requires */

const path = require("path");
const { app, BrowserWindow, ipcMain } = require("electron");
const serve = require("electron-serve");

const devUrl = "http://localhost:3000";
const indexHtmlDirPath = path.join(__dirname, "..", "..", "dist");

const loadUrl = serve({ directory: indexHtmlDirPath });

const createWindow = () => {
  const mainWindow = new BrowserWindow({
    width: 1200,
    height: 900,
    frame: false,
    show: false,
    webPreferences: {
      preload: path.join(__dirname, "preload.cjs"),
    },
  });

  mainWindow.maximize();

  mainWindow.show();

  ipcMain.on("main-window-toggle-maximize", () => {
    mainWindow.isMaximized() ? mainWindow.unmaximize() : mainWindow.maximize();
  });

  ipcMain.on("main-window-minimize", () => {
    mainWindow.minimize();
  });

  ipcMain.on("main-window-close", () => {
    mainWindow.close();
  });

  if (process.env.LOAD_DEV_URL === "true") {
    mainWindow.loadURL(devUrl).catch((error) => {
      console.error(
        "electron::start",
        `An error occurred while loading the ${devUrl} url: `,
        error
      );
    });
  } else {
    loadUrl(mainWindow).catch((error) => {
      console.error(
        "electron::start",
        `An error occurred while loading the index.html file located under ${indexHtmlDirPath}: `,
        error
      );
    });
  }
};

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app
  .whenReady()
  .then(() => {
    createWindow();

    app.on("activate", () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        createWindow();
      }
    });
  })
  .catch((error) => {
    console.error("electron::start", "Couldn't start the application: ", error);
  });
