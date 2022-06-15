export type Env = typeof envs[number];

const envs = ["local", "uat", "production"] as const;

export function getEnv({
  allowedApp,
  allowedDomain,
  url = new URL(window.location.href),
}: {
  allowedApp: string;
  allowedDomain: string;
  url?: URL;
}): Env | null {
  let env: Env;

  if (url.hostname === "localhost") {
    env = "local";
  } else {
    const parts = url.hostname.split(".");

    let app: string, domain: string, ext: string;

    if (parts.length === 3) {
      [app, domain, ext] = parts;
      env = "production";
    } else if (parts.length === 4) {
      [app, , domain, ext] = parts;

      if (!(envs as readonly string[]).includes(parts[1])) {
        return null;
      }

      env = parts[1] as Env;
    } else {
      return null;
    }

    if (app !== allowedApp || `${domain}.${ext}` !== allowedDomain) {
      return null;
    }
  }

  return env;
}
