import { createDevSettingsStore } from "@lgn/web-client/src/stores/devSettings";

export default createDevSettingsStore("dev-settings", {
  editorServerUrl: "http://[::1]:5051",
  runtimeServerUrl: "http://[::1]:5052",
});
