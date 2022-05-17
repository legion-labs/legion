import { createDevSettingsStore } from "@lgn/web-client/src/stores/devSettings";

export default createDevSettingsStore("dev-settings", {
  editorServerUrl: "http://[::1]:50051",
  runtimeServerUrl: "http://[::1]:50051",
});
