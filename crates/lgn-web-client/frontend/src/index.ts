import { authClient, initAuth, InitAuthStatus, LoginConfig } from "./lib/auth";
import log, { Level as LogLevel } from "./lib/log";
import userInfo from "./stores/userInfo";
import { SvelteComponentTyped } from "svelte";
import { grpc } from "@improbable-eng/grpc-web";

export class AppComponent extends SvelteComponentTyped<{
  initAuthStatus: InitAuthStatus | null;
  grpcMetadata: grpc.Metadata | null;
}> {}

export type AuthUserConfig = {
  /** The issuer url (i.e. the oauth provider url) */
  issuerUrl: string;
  /** The url to redirect the user to after they're logged in */
  redirectUri: string;
  /** The oauth client id */
  clientId: string;
  /** Login related configuration */
  login: LoginConfig;
  /**
   * Title used by the `history.replaceState` function,
   * [ignored](window.history.replaceState) for now.
   */
  redirectionTitle: string;
  /**
   * When set to `true` a new `grpcMetadata` prop is injected in the App component.
   * It can be used to access an API that requires auth.
   */
  grpc?: boolean;
};

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
  auth: AuthUserConfig | null;
  /** A valid query selector to mount your app into  */
  rootQuerySelector: string;
  /** Log level, if set to `null` logs are entirely disabled  */
  logLevel: LogLevel | null;
  /** Hook called before the application start */
  onPreInit?(): Promise<void> | void;
};

/**
 * Run a Legion client.
 * _Must be called at the beginning of any application that uses this library._
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
}: Config): Promise<void> {
  onPreInit && (await onPreInit());

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

  let grpcMetadata: grpc.Metadata | null = null;

  if (authConfig) {
    const { clientId, issuerUrl, redirectUri, login } = authConfig;

    initAuthStatus = await initAuth({
      clientId,
      issuerUrl,
      redirectUri,
      loginConfig: login,
    });

    if (authConfig.grpc) {
      const metadata = new grpc.Metadata();

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
