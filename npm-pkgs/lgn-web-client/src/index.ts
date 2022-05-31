import grpcWeb from "@improbable-eng/grpc-web";
import { SvelteComponentTyped } from "svelte";
import type { Unsubscriber } from "svelte/store";

import { initApiClient } from "./api";
import type { NonEmptyArray } from "./lib/array";
import type { InitAuthUserConfig } from "./lib/auth";
import { authClient, initAuth } from "./lib/auth";
import type { InitAuthStatus } from "./lib/auth";
import log from "./lib/log";
import type { Transport } from "./lib/log";
import userInfo from "./orchestrators/userInfo";

import "../../tailwind.css";

export class AppComponent extends SvelteComponentTyped<{
  dispose(): void;
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
  /** Hook called before the application start */
  onPreInit?(): Promise<void> | void;
  /** A Svelte component class */
  appComponent: typeof AppComponent;
  /** A valid query selector to mount your app into  */
  rootQuerySelector: string;
  // TODO: Improve type safety for `appComponent` and `extraProps` using
  // https://devblogs.microsoft.com/typescript/announcing-typescript-4-7-beta/#instantiation-expressions
  /** Optionally inject extra props to the app component */
  extraProps?: Record<string, unknown>;
  /**
   * Enables authentication.
   *
   * If authentication is not enabled some functionalities like `userInfo` will not be usable
   */
  auth?: InitAuthUserConfig;
  /** * Enables log */
  log?: {
    /** Transports to use */
    transports: NonEmptyArray<Transport>;
  };
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
  onPreInit,
  appComponent: AppComponent,
  extraProps,
  rootQuerySelector,
  auth: authConfig,
  log: logConfig,
  editorServerUrl,
  runtimeServerUrl,
}: Config): Promise<void> {
  await onPreInit?.();

  initApiClient({ editorServerUrl, runtimeServerUrl });

  const target = getTarget(rootQuerySelector);

  if (logConfig) {
    log.init(logConfig.transports);
  }

  if (!target) {
    return;
  }

  let initAuthStatus: InitAuthStatus | null = null;

  let grpcMetadata: grpcWeb.grpc.Metadata | null = null;

  let logUnsubscriber: Unsubscriber | null = null;

  if (authConfig) {
    initAuthStatus = await initAuth(authConfig);

    if (authConfig.grpc) {
      const metadata = new grpcWeb.grpc.Metadata();

      const token = authClient.accessToken;

      if (token) {
        metadata.set("Authorization", `Bearer ${token}`);
      } else {
        log.warn(
          "Couldn't build the grpc metadata object with auth, access token was not found"
        );
      }

      grpcMetadata = metadata;
    }

    if (logConfig) {
      logUnsubscriber = userInfo.data.subscribe((userInfo) => {
        log.debug(
          "user",
          userInfo
            ? log.json`User is authed: ${userInfo}`
            : "User is not authed"
        );
      });
    }
  }

  function dispose() {
    logUnsubscriber?.();

    if (logConfig) {
      log.dispose();
    }
  }

  try {
    new AppComponent({
      target,
      props: { ...extraProps, dispose, grpcMetadata, initAuthStatus },
    });
  } catch (error) {
    log.error(error);

    return;
  }
}

export type HeadlessConfig = {
  /** Hook called before the application start */
  onPreInit?(): Promise<void> | void;
  /**
   * Enables authentication.
   *
   * If authentication is not enabled some functionalities like `userInfo` will not be usable
   */
  auth?: InitAuthUserConfig;
  /** * Enables log */
  log?: {
    /** Transports to use */
    transports: NonEmptyArray<Transport>;
  };
  editorServerUrl?: string;
  runtimeServerUrl?: string;
};

export type HeadlessRun = {
  dispose(this: void): void;
  initAuthStatus: InitAuthStatus | null;
  grpcMetadata: grpcWeb.grpc.Metadata | null;
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
  onPreInit,
  auth: authConfig,
  log: logConfig,
  editorServerUrl,
  runtimeServerUrl,
}: HeadlessConfig): Promise<HeadlessRun> {
  await onPreInit?.();

  initApiClient({ editorServerUrl, runtimeServerUrl });

  if (logConfig) {
    log.init(logConfig.transports);
  }

  let initAuthStatus: InitAuthStatus | null = null;

  let grpcMetadata: grpcWeb.grpc.Metadata | null = null;

  let logUnsubscriber: Unsubscriber | null = null;

  if (authConfig) {
    initAuthStatus = await initAuth(authConfig);

    if (authConfig.grpc) {
      const metadata = new grpcWeb.grpc.Metadata();

      const token = authClient.accessToken;

      if (token) {
        metadata.set("Authorization", `Bearer ${token}`);
      } else {
        log.warn(
          "Couldn't build the grpc metadata object with auth, access token was not found"
        );
      }

      grpcMetadata = metadata;
    }

    if (logConfig) {
      logUnsubscriber = userInfo.data.subscribe((userInfo) => {
        log.debug(
          "user",
          userInfo
            ? log.json`User is authed: ${userInfo}`
            : "User is not authed"
        );
      });
    }
  }

  function dispose() {
    logUnsubscriber?.();

    if (logConfig) {
      log.dispose();
    }
  }

  return { initAuthStatus, grpcMetadata, dispose };
}
