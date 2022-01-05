import { getUserInfo as tauriGetUserInfo } from "./lib/auth/tauri";
import { userAuth as browserUserAuth } from "./lib/auth/browser";
import log, { Level as LogLevel } from "./lib/log";
import userInfo from "./stores/userInfo";
import { UserInfo } from "./lib/auth";

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

type HookLogOnlyArgs<HasLog extends boolean> = {
  log: HasLog extends true ? typeof log : null;
};

type HookLogOnlyFunction<HasLog extends boolean> = (
  args: HookLogOnlyArgs<HasLog>
) => void;

type HookArgs<HasLog extends boolean, HasAuth extends boolean> = {
  log: HasLog extends true ? typeof log : null;
  userInfo: HasAuth extends true ? UserInfo : null;
};

type HookFunction<HasLog extends boolean, HasAuth extends boolean> = (
  args: HookArgs<HasLog, HasAuth>
) => void;

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
}: Config<SvelteComponent>): Promise<void> {
  const target = getTarget(rootQuerySelector);

  if (logLevel) {
    log.init();
    log.set(logLevel);
  }

  if (!target) {
    return;
  }

  let userInfoSet: UserInfo | null = null;

  if (authConfig) {
    if (window.__TAURI__) {
      userInfoSet = await tauriGetUserInfo(userInfo, {
        forceAuth: authConfig.forceAuth,
      });
    } else {
      userInfoSet = await browserUserAuth(userInfo, {
        forceAuth: authConfig.forceAuth,
      });
    }
  }

  if (logLevel) {
    log.debug(
      "user",
      userInfoSet
        ? log.json`User is authed: ${userInfoSet}`
        : "User is not authed"
    );
  }

  try {
    new App({ target });
  } catch (error) {
    log.error(error);

    return;
  }
}
