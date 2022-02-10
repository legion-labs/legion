import { getUserInfo as tauriGetUserInfo } from "./lib/auth/tauri";
import { userAuth as browserUserAuth } from "./lib/auth/browser";
import log, { Level as LogLevel } from "./lib/log";
import userInfo from "./stores/userInfo";

export type AuthUserConfig = {
  /** Force authentication on application start */
  forceAuth: boolean;
  /** The path Cognito redirected the authed user to, typically `"/"` */
  redirectedTo: string;
  /** The path to redirect the user to after they got fully authenticated, typically `"/"` */
  redirectTo: string;
  /**
   * Title used by the `history.replaceState` function,
   * [ignored](window.history.replaceState) for now.
   */
  redirectionTitle: string;
};

export function defaultAuthUserConfig(): AuthUserConfig {
  return {
    forceAuth: false,
    redirectTo: "/",
    redirectedTo: "/",
    redirectionTitle: "Home",
  };
}

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

export type Config<SvelteComponent> = {
  /** A Svelte component class */
  appComponent: new (options: {
    target: Element | ShadowRoot;
  }) => SvelteComponent;
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
 * If the `forceAuth` option is `true` the unauthenticated users
 * will be redirected to Cognito.
 */
export async function run<SvelteComponent>({
  appComponent: App,
  auth: authConfig,
  rootQuerySelector,
  logLevel,
  onPreInit,
}: Config<SvelteComponent>): Promise<void> {
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

  if (authConfig) {
    if (window.__TAURI__) {
      await tauriGetUserInfo({
        forceAuth: authConfig.forceAuth,
      });
    } else {
      await browserUserAuth({
        forceAuth: authConfig.forceAuth,
      });
    }
  }

  try {
    new App({ target });
  } catch (error) {
    log.error(error);

    return;
  }
}
