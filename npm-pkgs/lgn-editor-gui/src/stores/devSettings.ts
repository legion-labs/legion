import { createDevSettingsStore } from "@lgn/web-client/src/stores/devSettings";

export default createDevSettingsStore("dev-settings", {
  grpcEditorServerUrl: "http://[::1]:50051",
  grpcRuntimeServerUrl: "http://[::1]:50052",
  restEditorServerUrl: "http://[::1]:5051",
  restRuntimeServerUrl: "http://[::1]:5052",
});
