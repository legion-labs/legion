import { authClient, initAuth, InitAuthUserConfig } from "./lib/auth";
import type { InitAuthStatus } from "./lib/auth";
import log from "./lib/log";
import type { Level as LogLevel } from "./lib/log";
import userInfo from "./orchestrators/userInfo";
import { SvelteComponentTyped } from "svelte";
import grpcWeb from "@improbable-eng/grpc-web";
import { initApiClient } from "./api";

export class AppComponent extends SvelteComponentTyped<{
  initAuthStatus: InitAuthStatus | null;
  grpcMetadata: grpcWeb.grpc.Metadata | null;
}> {}

/**
 * Find the root element
 * @param rootQuerySelector A valid query selector that targets the root element
 */
export function getTarget(rootQuerySelector: string) {
  const target = document.querySelector("#root");

  if (!target) {
    log.error(`${rootQuerySelector} element can't be found`);

    return null;
  }

  return target;
}

export type Config = {
  /** A Svelte component class */
  appComponent: typeof AppComponent;
  /**
   * Enable authentication or not (using `null`).
   *
   * If authentication is not enabled some functionalities like `userInfo` will not be usable
   */
  auth: InitAuthUserConfig | null;
  /** A valid query selector to mount your app into  */
  rootQuerySelector: string;
  /** Log level, if set to `null` logs are entirely disabled  */
  logLevel: LogLevel | null;
  /** Hook called before the application start */
  onPreInit?(): Promise<void> | void;
  editorServerUrl?: string;
  runtimeServerUrl?: string;
};

/**
 * Run a Legion client.
 * _Must be called **once** at the beginning of any application that uses this library._
 *
 * This function will inject the following props into the provided `appComponent`:
 * - `initAuthStatus`: can contain an `authorizationUrl` if auth failed.
 *     This url must be used to redirect the user.
 *     This has value `null` if `auth` config is not set.
 * - `grpcMetadata`: contains a grpc `Metadata` object ready for auth.
 *     This has value `null` if `auth` config is not set.
 */
export async function run({
  appComponent: AppComponent,
  auth: authConfig,
  rootQuerySelector,
  logLevel,
  onPreInit,
  editorServerUrl,
  runtimeServerUrl,
}: Config): Promise<void> {
  onPreInit && (await onPreInit());

  initApiClient({ editorServerUrl, runtimeServerUrl });

  const target = getTarget(rootQuerySelector);

  if (logLevel) {
    log.init();
    log.set(logLevel);

    userInfo.data.subscribe((userInfo) => {
      log.debug(
        "user",
        userInfo ? log.json`User is authed: ${userInfo}` : "User is not authed"
      );
    });
  }

  if (!target) {
    return;
  }

  let initAuthStatus: InitAuthStatus | null = null;

  let grpcMetadata: grpcWeb.grpc.Metadata | null = null;

  if (authConfig) {
    initAuthStatus = await initAuth(authConfig);

    if (authConfig.grpc) {
      const metadata = new grpcWeb.grpc.Metadata();

      const token = authClient.accessToken;

      if (!token) {
        log.warn(
          "Couldn't build the grpc metadata object with auth, access token was not found"
        );
      }

      metadata.set("Authorization", `Bearer ${token}`);

      grpcMetadata = metadata;
    }
  }

  try {
    new AppComponent({ target, props: { grpcMetadata, initAuthStatus } });
  } catch (error) {
    log.error(error);

    return;
  }
}

export type HeadlessConfig = {
  /**
   * Enable authentication or not (using `null`).
   *
   * If authentication is not enabled some functionalities like `userInfo` will not be usable
   */
  auth: InitAuthUserConfig | null;
  /** Log level, if set to `null` logs are entirely disabled  */
  logLevel: LogLevel | null;
  /** Hook called before the application start */
  onPreInit?(): Promise<void> | void;
  editorServerUrl?: string;
  runtimeServerUrl?: string;
};

/**
 * Alternative to the `run` function that doesn't start a Svelte application.
 * Typically used with a SvelteKit application.
 * _Must be called **once** at the beginning of any application that uses this library._
 *
 * This function returns the following values:
 * - `initAuthStatus`: can contain an `authorizationUrl` if auth failed.
 *     This url must be used to redirect the user.
 *     This has value `null` if `auth` config is not set.
 * - `grpcMetadata`: contains a grpc `Metadata` object ready for auth.
 *     This has value `null` if `auth` config is not set.
 */
export async function headlessRun({
  auth: authConfig,
  logLevel,
  onPreInit,
  editorServerUrl,
  runtimeServerUrl,
}: HeadlessConfig): Promise<{
  initAuthStatus: InitAuthStatus | null;
  grpcMetadata: grpcWeb.grpc.Metadata | null;
}> {
  onPreInit && (await onPreInit());

  initApiClient({ editorServerUrl, runtimeServerUrl });

  if (logLevel) {
    log.init();
    log.set(logLevel);

    userInfo.data.subscribe((userInfo) => {
      log.debug(
        "user",
        userInfo ? log.json`User is authed: ${userInfo}` : "User is not authed"
      );
    });
  }

  let initAuthStatus: InitAuthStatus | null = null;

  let grpcMetadata: grpcWeb.grpc.Metadata | null = null;

  if (authConfig) {
    initAuthStatus = await initAuth(authConfig);

    if (authConfig.grpc) {
      const metadata = new grpcWeb.grpc.Metadata();

      const token = authClient.accessToken;

      if (!token) {
        log.warn(
          "Couldn't build the grpc metadata object with auth, access token was not found"
        );
      }

      metadata.set("Authorization", `Bearer ${token}`);

      grpcMetadata = metadata;
    }
  }

  return { initAuthStatus, grpcMetadata };
}
