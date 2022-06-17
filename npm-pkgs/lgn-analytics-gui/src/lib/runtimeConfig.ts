import { env } from "@/constants";

import runtimeConfigs from "../config.json";

export type RuntimeConfig = Exclude<ReturnType<typeof getRuntimeConfig>, null>;

const defaultRuntimeConfig = {
  // We don't throw an error to not block the app from rendering (yet)
  clientId: "",
  cognitoPoolId: "",
  cognitoRegion: "",
  apiAnalytics: {
    host:
      typeof import.meta.env.VITE_LEGION_ANALYTICS_REMOTE_HOST === "string"
        ? import.meta.env.VITE_LEGION_ANALYTICS_REMOTE_HOST
        : "",
    url:
      typeof import.meta.env.VITE_LEGION_ANALYTICS_API_URL === "string"
        ? import.meta.env.VITE_LEGION_ANALYTICS_API_URL
        : "",
  },
};

export function getRuntimeConfig() {
  if (!env) {
    return defaultRuntimeConfig;
  }

  const config = runtimeConfigs[env];

  if (config === undefined) {
    return defaultRuntimeConfig;
  }

  return {
    ...config,
    apiAnalytics: {
      host:
        typeof import.meta.env.VITE_LEGION_ANALYTICS_REMOTE_HOST === "string"
          ? import.meta.env.VITE_LEGION_ANALYTICS_REMOTE_HOST
          : config.apiAnalytics.host,
      url:
        typeof import.meta.env.VITE_LEGION_ANALYTICS_API_URL === "string"
          ? import.meta.env.VITE_LEGION_ANALYTICS_API_URL
          : config.apiAnalytics.url,
    },
  };
}
