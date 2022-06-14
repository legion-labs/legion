import { getRuntimeConfig as genericGetRuntimeConfig } from "@lgn/web-client/src/lib/runtimeConfig";

import runtimeConfigs from "../config.json";

export type RuntimeConfig = Exclude<ReturnType<typeof getRuntimeConfig>, null>;

export function getRuntimeConfig() {
  return (
    genericGetRuntimeConfig({
      allowedApp: "analytics",
      allowedDomain: "legionengine.com",
      configs: runtimeConfigs,
    }) ?? {
      // We don't throw an error to not block the app from rendering (yet)
      clientId: "",
      cognitoPoolId: "",
      cognitoRegion: "",
    }
  );
}
