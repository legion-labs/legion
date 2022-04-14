import path from "path";
import { app, BrowserWindow, ipcMain } from "electron";
import serve from "electron-serve";

if (!process.env["ELECTRON_RUNTIME_CONFIGURATION"]) {
  console.error(
    "ELECTRON_RUNTIME_CONFIGURATION environment variable is not set"
  );

  process.exit(1);
}

const configuration = JSON.parse(process.env["ELECTRON_RUNTIME_CONFIGURATION"]);

const loadUrl = serve({ directory: configuration.source.value });

function createWindow() {
  const mainWindow = new BrowserWindow({
    width: configuration.dimension[0],
    height: configuration.dimension[1],
    frame: ["full", "topbarOnly"].includes(configuration.decoration),
    show: false,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
    },
  });

  if (configuration.fullscreen) {
    mainWindow.maximize();
  }

  if (["none", "topbarOnly"].includes(configuration.decoration)) {
    mainWindow.removeMenu();
  }

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

  if (configuration.source.type === "remote") {
    mainWindow.loadURL(configuration.source.value).catch((error) => {
      console.error(
        "electron::start",
        `An error occured while loading the ${configuration.source.value} url: `,
        error
      );
    });
  } else {
    loadUrl(mainWindow).catch((error) => {
      console.error(
        "electron::start",
        `An error occured while loading the index.html file located under ${configuration.source.value}: `,
        error
      );
    });
  }
}

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
