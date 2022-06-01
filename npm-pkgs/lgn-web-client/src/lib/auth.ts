import getPkce from "oauth-pkce";

import userInfo from "../orchestrators/userInfo";
import accessToken from "../stores/accessToken";
import type { NonEmptyArray } from "./array";
import { getCookie, setCookie } from "./cookie";
import { displayError } from "./errors";
import log from "./log";

// We check each hour if the refresh token should be refreshed
const refreshTokenTimeThreshold = 60 * 60 * 1_000;

// https://connect2id.com/products/server/docs/api/token#token-response
export type ClientTokenSet = {
  [key: string]: unknown;
  access_token: string;
  token_type: "Bearer" | "DPoP";
  expires_in: number;
  refresh_token?: string;
};

// https://openid.net/specs/openid-connect-discovery-1_0.html#ProviderConfig
const suffix = "/.well-known/openid-configuration";

// https://openid.net/specs/openid-connect-discovery-1_0.html#ProviderConfigurationResponse
class IssuerConfiguration {
  config: {
    issuer: string;
    authorization_endpoint: string;
    /* REQUIRED only in implicit flow */ token_endpoint?: string;
    /* RECOMMENDED */ userinfo_endpoint?: string;
    jwks_uri: string;
    /* RECOMMENDED */ registration_endpoint: string;
    /* RECOMMENDED */ scopes_supported: string[];
    response_types_supported: string[];
    response_modes_supported?: string[];
    grant_types_supported?: string[];
    acr_values_supported?: string[];
    subject_types_supported: ("pairwise" | "public")[];
    id_token_signing_alg_values_supported: string[];
    id_token_encryption_alg_values_supported?: string[];
    id_token_encryption_enc_values_supported?: string[];
    userinfo_signing_alg_values_supported?: string[];
    userinfo_encryption_alg_values_supported?: string[];
    userinfo_encryption_enc_values_supported?: string[];
    request_object_signing_alg_values_supported?: string[];
    request_object_encryption_alg_values_supported?: string[];
    request_object_encryption_enc_values_supported?: string[];
    token_endpoint_auth_methods_supported?: string[];
    token_endpoint_auth_signing_alg_values_supported?: string[];
    display_values_supported?: string[];
    claim_types_supported?: string[];
    /* RECOMMENDED */ claims_supported?: string[];
    service_documentation?: string;
    claims_locales_supported?: string[];
    ui_locales_supported?: string[];
    claims_parameter_supported?: boolean;
    request_parameter_supported?: boolean;
    request_uri_parameter_supported?: boolean;
    require_request_uri_registration?: boolean;
    op_policy_uri?: string;
    op_tos_uri?: string;
  };

  constructor(config: IssuerConfiguration["config"]) {
    this.config = config;
  }

  static async fromString(
    url: string,
    fetch = globalThis.fetch.bind(globalThis)
  ) {
    const configResponse = await fetch(url + suffix);
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const config: IssuerConfiguration["config"] = await configResponse.json();

    return new IssuerConfiguration(config);
  }

  static async fromUrl(url: URL, fetch = globalThis.fetch.bind(globalThis)) {
    return IssuerConfiguration.fromString(url.toString(), fetch);
  }
}

export class CookieStorage {
  accessTokenName: string;
  refreshTokenName: string;
  expiresAtName: string;

  constructor({
    accessTokenName,
    refreshTokenName,
    expiresAtName,
  }: {
    accessTokenName?: string;
    refreshTokenName?: string;
    expiresAtName?: string;
  } = {}) {
    this.accessTokenName = accessTokenName || "access_token";
    this.refreshTokenName = refreshTokenName || "refresh_token";
    this.expiresAtName = expiresAtName || "expires_at";
  }

  // The refresh token arbitraly last for 1 day by default
  store(
    // eslint-disable-next-line camelcase
    { access_token, expires_in, refresh_token }: ClientTokenSet,
    refreshTokenExpiresIn = 24 * 60 * 60
  ) {
    setCookie(this.accessTokenName, access_token, expires_in);

    // eslint-disable-next-line camelcase
    setCookie(this.expiresAtName, Date.now() + expires_in * 1_000, expires_in);

    // eslint-disable-next-line camelcase
    if (refresh_token) {
      setCookie(this.refreshTokenName, refresh_token, refreshTokenExpiresIn);
    }
  }

  get accessToken() {
    return getCookie(this.accessTokenName);
  }

  get refreshToken() {
    return getCookie(this.refreshTokenName);
  }

  get expiresAt() {
    const expiresAt = getCookie(this.expiresAtName);

    if (expiresAt === null) {
      return null;
    }

    const expiresAtValue = +expiresAt;

    if (isNaN(expiresAtValue)) {
      return null;
    }

    return new Date(expiresAtValue);
  }
}

export type LoginConfig = {
  scopes: NonEmptyArray<string>;
  extraParams?: Record<string, string>;
  popupTitle?: string;
  redirectUri?: URL;
  cookies?: {
    accessToken?: string;
    refreshToken?: string;
  };
};

class Client<UserInfo> {
  protected clientId: string;
  protected issuerConfiguration: IssuerConfiguration;
  protected fetch: typeof globalThis.fetch;
  protected redirectUri?: URL;

  constructor(
    issuerConfiguration: IssuerConfiguration,
    clientId: string,
    {
      fetch = globalThis.fetch.bind(globalThis),
      redirectUri,
    }: { fetch?: typeof globalThis.fetch; redirectUri?: string | URL } = {}
  ) {
    this.clientId = clientId;
    this.issuerConfiguration = issuerConfiguration;
    this.redirectUri =
      redirectUri instanceof URL
        ? redirectUri
        : typeof redirectUri === "string"
        ? new URL(redirectUri)
        : undefined;
    this.fetch = fetch;
  }

  authorizeUrl({
    responseType,
    scopes,
    extraParams,
    redirectUri = this.redirectUri,
    pkceChallenge,
  }: {
    responseType: string;
    scopes: NonEmptyArray<string>;
    extraParams?: Record<string, string>;
    redirectUri?: string | URL;
    pkceChallenge?: string;
  }) {
    const authorizationUrl = new URL(
      this.issuerConfiguration.config.authorization_endpoint
    );

    if (
      !this.issuerConfiguration.config.response_types_supported.includes(
        responseType
      )
    ) {
      log.warn(
        `Unsupported response type ${responseType}, supported response types are ${this.issuerConfiguration.config.response_types_supported.join(
          ", "
        )}`
      );
    }

    for (const scope of scopes) {
      if (!this.issuerConfiguration.config.scopes_supported.includes(scope)) {
        log.warn(
          `Unsupported scope ${scope}, supported scopes are ${this.issuerConfiguration.config.scopes_supported.join(
            ", "
          )}`
        );
      }
    }

    if (pkceChallenge) {
      authorizationUrl.searchParams.set("code_challenge_method", "S256");
      authorizationUrl.searchParams.set("code_challenge", pkceChallenge);
    }

    authorizationUrl.searchParams.set("client_id", this.clientId);
    authorizationUrl.searchParams.set("response_type", responseType);
    authorizationUrl.searchParams.set("scope", scopes.join(" "));

    // TODO: Check strings length > 0
    if (redirectUri) {
      authorizationUrl.searchParams.set("redirect_uri", redirectUri.toString());
    }

    if (extraParams) {
      for (const extraParamKey in extraParams) {
        authorizationUrl.searchParams.set(
          extraParamKey,
          extraParams[extraParamKey]
        );
      }
    }

    return authorizationUrl.toString();
  }

  async exchangeRefreshTokenRequest(
    refreshToken: string
  ): Promise<ClientTokenSet> {
    if (!this.issuerConfiguration.config.token_endpoint) {
      throw new Error("Token endpoint not specified by provider");
    }

    const body = new URLSearchParams({
      // eslint-disable-next-line camelcase
      grant_type: "refresh_token",
      // eslint-disable-next-line camelcase
      client_id: this.clientId,
      // eslint-disable-next-line camelcase
      refresh_token: refreshToken,
    });

    const requestInit: RequestInit = {
      method: "POST",
      mode: "cors",
      body,
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
    };

    const response = await this.fetch(
      this.issuerConfiguration.config.token_endpoint,
      requestInit
    );

    if (!response.ok) {
      throw new Error(await response.text());
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    return response.json();
  }

  async exchangeCode(
    code: string,
    {
      pkceVerifier,
      redirectUri = this.redirectUri,
    }: { pkceVerifier?: string; redirectUri?: string | URL } = {}
  ) {
    if (!this.issuerConfiguration.config.token_endpoint) {
      throw new Error("Token endpoint not specified by provider");
    }

    if (!redirectUri) {
      throw new Error("No redirect uri specified");
    }

    const body = new URLSearchParams({
      // eslint-disable-next-line camelcase
      grant_type: "authorization_code",
      // eslint-disable-next-line camelcase
      client_id: this.clientId,
      code,
      // eslint-disable-next-line camelcase
      redirect_uri: redirectUri.toString(),
    });

    if (pkceVerifier) {
      body.set("code_verifier", pkceVerifier);
    }

    const requestInit: RequestInit = {
      method: "POST",
      mode: "cors",
      body,
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
    };

    const response = await this.fetch(
      this.issuerConfiguration.config.token_endpoint,
      requestInit
    );

    if (!response.ok) {
      throw new Error(await response.text());
    }

    return response.json();
  }

  async userInfo(accessToken: string): Promise<UserInfo> {
    if (!this.issuerConfiguration.config.userinfo_endpoint) {
      throw new Error("User info endpoint not specified by provider");
    }

    const requestInit: RequestInit = {
      method: "GET",
      mode: "cors",
      headers: { Authorization: `Bearer ${accessToken}` },
    };

    const request = new Request(
      this.issuerConfiguration.config.userinfo_endpoint,
      requestInit
    );

    const response = await this.fetch(request);

    if (!response.ok) {
      throw new Error(await response.text());
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    return response.json();
  }
}

class PkceChallenge {
  static async newRandomSha256() {
    return new Promise<{ challenge: string; verifier: string }>(
      (resolve, reject) => {
        getPkce(43, (error, { verifier, challenge }) => {
          if (error) {
            return reject(error);
          }

          resolve({ verifier, challenge });
        });
      }
    );
  }
}

// "Userland" - Code related to the Legion applications

export type UserInfo = {
  sub: string;
  name?: string;
  given_name?: string;
  family_name?: string;
  middle_name?: string;
  nickname?: string;
  preferred_username?: string;
  profile?: string;
  picture?: string;
  website?: string;
  email?: string;
  email_verified?: "true" | "false";
  gender?: string;
  birthdate?: string;
  zoneinfo?: string;
  locale?: string;
  phone_number?: string;
  phone_number_verified?: "true" | "false";
  updated_at?: string;
  // Azure-specific fields.
  //
  // This is a merely a convention, but we need one.
  //
  // These fields contains the Azure-specific information about the user, which allow us to query
  // the Azure API for extended user information (like the user's photo).
  "custom:azure_oid"?: string;
  "custom:azure_tid"?: string;
};

export class LegionClient extends Client<UserInfo> {
  loginConfig: LoginConfig;
  #cookieStorage: CookieStorage;
  #authorizeVerifierStorageKey = "authorize-verifier";
  #targetUrlStorageKey = "target-url";

  constructor(
    issuerConfiguration: IssuerConfiguration,
    clientId: string,
    config: { fetch?: typeof globalThis.fetch; redirectUri?: string | URL },
    loginConfig: LoginConfig
  ) {
    super(issuerConfiguration, clientId, config);

    this.loginConfig = loginConfig;
    this.#cookieStorage = new CookieStorage({
      accessTokenName: this.loginConfig.cookies?.accessToken,
      refreshTokenName: this.loginConfig.cookies?.refreshToken,
    });
  }

  get accessToken() {
    return this.#cookieStorage.accessToken;
  }

  get refreshToken() {
    return this.#cookieStorage.refreshToken;
  }

  get redirectUris() {
    return {
      login: this.loginConfig.redirectUri || this.redirectUri,
    };
  }

  async refreshClientTokenSet(): Promise<ClientTokenSet> {
    if (!this.refreshToken) {
      throw new Error("Refresh token not found");
    }

    const clientTokenSet = await this.exchangeRefreshTokenRequest(
      this.refreshToken
    );

    accessToken.set(clientTokenSet.access_token);

    return clientTokenSet;
  }

  async getAuthorizationUrl() {
    const { challenge, verifier } = await PkceChallenge.newRandomSha256();

    const authorizeUrl = authClient.authorizeUrl({
      responseType: "code",
      scopes: this.loginConfig.scopes,
      extraParams: this.loginConfig.extraParams,
      pkceChallenge: challenge,
    });

    localStorage.setItem(this.#authorizeVerifierStorageKey, verifier);

    if (location.origin === authClient.redirectUri?.origin) {
      localStorage.setItem(this.#targetUrlStorageKey, location.href);
    }

    return authorizeUrl;
  }

  async startTokenSetAutoRefresh() {
    const expiresAt = this.#cookieStorage.expiresAt;

    if (!expiresAt) {
      throw new Error(
        "Couldn't start token set auto refresh, expires in cookie is not set or not valid"
      );
    }

    if (expiresAt.getTime() - Date.now() < refreshTokenTimeThreshold) {
      authClient.storeClientTokenSet(await this.refreshClientTokenSet());
    }

    setTimeout(() => {
      this.startTokenSetAutoRefresh().catch((error) => {
        log.warn("auth", `Couldn't refresh token set: ${displayError(error)}`);
      });
    }, refreshTokenTimeThreshold);
  }

  getTargetUrl(): URL | null {
    const target = localStorage.getItem(this.#targetUrlStorageKey);

    localStorage.removeItem(this.#targetUrlStorageKey);

    if (!target) {
      return null;
    }

    const targetUrl = new URL(target);

    if (location.origin !== targetUrl.origin) {
      return null;
    }

    return targetUrl;
  }

  async getClientTokenSet(url: URL | string): Promise<ClientTokenSet | null> {
    const parsedUrl = url instanceof URL ? url : new URL(url);

    const searchParams = new URLSearchParams(parsedUrl.search);

    const code = searchParams.get("code");

    if (!code) {
      return null;
    }

    if (!this.redirectUris.login) {
      throw new Error("No redirect uri specified");
    }

    const verifier = localStorage.getItem(this.#authorizeVerifierStorageKey);

    localStorage.removeItem(this.#authorizeVerifierStorageKey);

    if (!verifier) {
      throw new Error("Couldn't find verifier in storage");
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const clientTokenSet: ClientTokenSet | null = await authClient.exchangeCode(
      code,
      {
        pkceVerifier: verifier,
      }
    );

    if (!clientTokenSet) {
      throw new Error("No client token set returned by the provider");
    }

    accessToken.set(clientTokenSet.access_token);

    return clientTokenSet;
  }

  override async userInfo(): Promise<UserInfo> {
    const accessToken =
      globalThis.isElectron && globalThis.electron
        ? globalThis.electron.auth.getAccessToken()
        : getCookie(this.#cookieStorage.accessTokenName);

    if (!accessToken) {
      throw new Error("Access token not found");
    }

    return super.userInfo(accessToken);
  }

  storeClientTokenSet(clientTokenSet: ClientTokenSet) {
    this.#cookieStorage.store(clientTokenSet);
  }
}

export type InitAuthStatus =
  // User is authed or could be authed
  | { type: "success" }
  // User is
  | { type: "error"; authorizationUrl: string };

export let authClient: LegionClient;

export type InitAuthUserConfig = {
  /** The issuer url (i.e. the oauth provider url) */
  issuerUrl: string;
  /** The url to redirect the user to after they're logged in */
  redirectUri: string;
  /** The oauth client id */
  clientId: string;
  /** Login related configuration */
  login: LoginConfig;
  /**
   * When set to `true` a new `grpcMetadata` prop is injected in the App component.
   * It can be used to access an API that requires auth.
   */
  grpc?: boolean;
  /** Overrides the `fetch` function */
  fetch?: typeof globalThis.fetch;
  /** The current url to read code from, defaults to `globalThis.location` */
  url?: URL | Location;
  /**
   * Function used after the user is logged and is redirected to the provided `redirectUri`
   * Defaults to `globalThis.history.replaceState`.
   *
   * The `url` argument will have the same value as `InitAuthUserConfig.url`.
   *
   * If you provide your own function it's strongly adviced to use an alternative that's close
   * to  `globalThis.history.replaceState` with history state replacement.
   */
  redirectFunction?: (url: URL) => Promise<void> | void;
};

export async function initAuth({
  issuerUrl,
  clientId,
  redirectUri,
  login,
  redirectFunction,
  fetch = globalThis.fetch.bind(globalThis),
  url = globalThis.location,
}: InitAuthUserConfig): Promise<InitAuthStatus> {
  // Initialize the auth client
  if (!authClient) {
    const issuerConfiguration = await IssuerConfiguration.fromString(
      issuerUrl,
      fetch
    );

    const client = new LegionClient(
      issuerConfiguration,
      clientId,
      { fetch, redirectUri },
      login
    );

    authClient = client;
  }

  if (globalThis.isElectron === true && globalThis.electron) {
    await globalThis.electron.auth.initOAuthClient();

    try {
      await userInfo.run(async () => {
        if (!globalThis.electron) {
          // Should never happen
          throw new Error("Not in Electron");
        }

        const userInfo = await globalThis.electron?.auth.authenticate(
          authClient.loginConfig.scopes,
          authClient.loginConfig.extraParams
        );

        log.debug("auth", userInfo);

        return userInfo;
      });
    } catch {
      return {
        type: "error",
        authorizationUrl: await authClient.getAuthorizationUrl(),
      };
    }

    accessToken.set(authClient.accessToken);

    return { type: "success" };
  }

  // Try to get the code from the url, if present and an error occurs
  // we assume the user is not logged in properly and must be redirected to the authorize url
  try {
    let targetUrl: URL | null = null;

    try {
      targetUrl = authClient.getTargetUrl();
    } catch {
      // Ignored
    } finally {
      targetUrl = targetUrl || authClient.redirectUris.login || null;
    }

    const clientTokenSet = await authClient.getClientTokenSet(url.href);

    if (clientTokenSet) {
      authClient.storeClientTokenSet(clientTokenSet);

      if (targetUrl) {
        log.debug(`Redirecting to ${targetUrl.toString()}`);

        if (redirectFunction) {
          await redirectFunction(targetUrl);
        } else {
          globalThis.history.replaceState(null, "", targetUrl);
        }
      }
    }
  } catch (error) {
    log.warn(
      `An error occured while trying to get the client token set ${displayError(
        error
      )}`
    );

    return {
      type: "error",
      authorizationUrl: await authClient.getAuthorizationUrl(),
    };
  }

  // Normal workflow, no code in the url, we let the application
  // know that the auth is not done at all
  if (!authClient.accessToken && !authClient.refreshToken) {
    return {
      type: "error",
      authorizationUrl: await authClient.getAuthorizationUrl(),
    };
  }

  // We can silently refresh the client token set if a refresh token is present
  if (!authClient.accessToken && authClient.refreshToken) {
    try {
      authClient.storeClientTokenSet(await authClient.refreshClientTokenSet());
    } catch (error) {
      log.warn(
        log.json`An error occured while trying to refresh the client token set ${error}`
      );

      return {
        type: "error",
        authorizationUrl: await authClient.getAuthorizationUrl(),
      };
    }
  }

  // Populate the user info store
  // At that point this request should not fail
  try {
    await userInfo.run(() => authClient.userInfo());
  } catch (error) {
    log.warn(
      log.json`An error occured while trying to get the user info ${error}`
    );

    return {
      type: "error",
      authorizationUrl: await authClient.getAuthorizationUrl(),
    };
  }

  try {
    await authClient.startTokenSetAutoRefresh();
  } catch (error) {
    log.warn(
      log.json`An error occured while starting the token set auto refresh ${error}`
    );

    return {
      type: "error",
      authorizationUrl: await authClient.getAuthorizationUrl(),
    };
  }

  accessToken.set(authClient.accessToken);

  // All good
  return { type: "success" };
}
