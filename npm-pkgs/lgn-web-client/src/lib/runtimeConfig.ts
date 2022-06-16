import type { Env } from "./env";
import { getEnv } from "./env";

export function getRuntimeConfig<
  RuntimeConfig extends Record<string, unknown>
>({
  allowedApp,
  allowedDomain,
  configs,
  url = new URL(window.location.href),
}: {
  allowedApp: string;
  allowedDomain: string;
  configs: Record<Env, RuntimeConfig>;
  url?: URL;
}): RuntimeConfig | null {
  const env = getEnv({ allowedApp, allowedDomain, url });

  if (env === null) {
    return null;
  }

  return configs[env];
}
