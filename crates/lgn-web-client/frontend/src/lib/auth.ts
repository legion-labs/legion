import userInfo from "../stores/userInfo";
import { getCookie, setCookie } from "./cookie";
import log from "./log";
import getPkce from "oauth-pkce";

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

  static async fromString(url: string) {
    const configResponse = await fetch(url + suffix);
    const config = await configResponse.json();

    return new IssuerConfiguration(config);
  }

  static async fromUrl(url: URL) {
    return IssuerConfiguration.fromString(url.toString());
  }
}

export class CookieStorage {
  accessTokenName: string;
  refreshTokenName: string;

  constructor({
    accessTokenName,
    refreshTokenName,
  }: { accessTokenName?: string; refreshTokenName?: string } = {}) {
    this.accessTokenName = accessTokenName || "access_token";
    this.refreshTokenName = refreshTokenName || "refresh_token";
  }

  // The refresh token arbitraly last for 1 day by default
  store(
    { access_token, expires_in, refresh_token }: ClientTokenSet,
    refreshTokenExpiresIn = 24 * 60 * 60
  ) {
    setCookie(this.accessTokenName, access_token, expires_in);

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
}

export type LoginConfig = {
  scopes: [string, ...string[]];
  extraParams?: Record<string, string>;
  popupTitle?: string;
  redirectUri?: string;
  cookies?: {
    accessToken?: string;
    refreshToken?: string;
  };
};

class Client<UserInfo> {
  protected clientId: string;
  protected issuerConfiguration: IssuerConfiguration;
  protected config: { redirectUri?: string };

  constructor(
    issuerConfiguration: IssuerConfiguration,
    clientId: string,
    config: { redirectUri?: string }
  ) {
    this.clientId = clientId;
    this.issuerConfiguration = issuerConfiguration;
    this.config = config;
  }

  authorizeUrl({
    responseType,
    scopes,
    extraParams,
    redirectUri = this.config.redirectUri,
    pkceChallenge,
  }: {
    responseType: string;
    scopes: [string, ...string[]];
    extraParams?: Record<string, string>;
    redirectUri?: string;
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
      authorizationUrl.searchParams.set("redirect_uri", redirectUri);
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
      grant_type: "refresh_token",
      client_id: this.clientId,
      refresh_token: refreshToken,
    });

    const requestInit: RequestInit = {
      method: "POST",
      mode: "cors",
      body,
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
    };

    const response = await fetch(
      new Request(this.issuerConfiguration.config.token_endpoint, requestInit)
    );

    if (!response.ok) {
      throw new Error(await response.text());
    }

    return response.json();
  }

  async exchangeCode(
    code: string,
    {
      pkceVerifier,
      redirectUri = this.config.redirectUri,
    }: { pkceVerifier?: string; redirectUri?: string } = {}
  ) {
    if (!this.issuerConfiguration.config.token_endpoint) {
      throw new Error("Token endpoint not specified by provider");
    }

    if (!redirectUri) {
      throw new Error("No redirect uri specified");
    }

    const body = new URLSearchParams({
      grant_type: "authorization_code",
      client_id: this.clientId,
      code,
      redirect_uri: redirectUri,
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

    const response = await fetch(
      new Request(this.issuerConfiguration.config.token_endpoint, requestInit)
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

    const response = await fetch(request);

    if (!response.ok) {
      throw new Error(await response.text());
    }

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

  constructor(
    issuerConfiguration: IssuerConfiguration,
    clientId: string,
    config: { redirectUri?: string },
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
      login: this.loginConfig.redirectUri || this.config.redirectUri,
    };
  }

  async refreshClientTokenSet(): Promise<ClientTokenSet> {
    if (!this.refreshToken) {
      throw new Error("Refresh token not found");
    }

    return this.exchangeRefreshTokenRequest(this.refreshToken);
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

    return authorizeUrl;
  }

  async getClientTokenSet(url: URL | string): Promise<ClientTokenSet | null> {
    if (window.__TAURI_METADATA__) {
      return null;
    }

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

    const clientTokenSet = await authClient.exchangeCode(code, {
      pkceVerifier: verifier,
    });

    if (!clientTokenSet) {
      throw new Error("No client token set returned by the provider");
    }

    return clientTokenSet;
  }

  override async userInfo(): Promise<UserInfo> {
    let accessToken: string | null = null;

    if (window.__TAURI_METADATA__) {
      const { invoke } = await import("@tauri-apps/api");
      accessToken = await invoke("plugin:browser|get_access_token");
    } else {
      accessToken = getCookie(this.#cookieStorage.accessTokenName);
    }

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

export async function initAuth({
  issuerUrl,
  clientId,
  redirectUri,
  loginConfig,
  force = false,
}: {
  issuerUrl: string;
  clientId: string;
  redirectUri: string;
  loginConfig: LoginConfig;
  force?: boolean;
}): Promise<InitAuthStatus> {
  // Initialize the auth client
  if (!authClient || force) {
    const issuerConfiguration = await IssuerConfiguration.fromString(issuerUrl);

    const client = new LegionClient(
      issuerConfiguration,
      clientId,
      { redirectUri },
      loginConfig
    );

    authClient = client;
  }

  // Tauri has its own way to deal with auth
  if (window.__TAURI_METADATA__) {
    try {
      await userInfo.run(async () => {
        const { invoke } = await import("@tauri-apps/api");

        const userInfo = (await invoke("plugin:browser|authenticate", {
          scopes: authClient.loginConfig.scopes,
          extraParams: authClient.loginConfig.extraParams,
        })) as UserInfo;

        log.debug("auth", userInfo);

        return userInfo;
      });
    } catch {
      // Nothing we can do about this but warn the user
      log.error("Couldn't authenticate the user");
    }

    return { type: "success" };
  }

  // Try to get the code from the url, if present and an error occurs
  // we assume the user is not logged in properly and must be redirected to the authorize url
  try {
    const clientTokenSet = await authClient.getClientTokenSet(
      window.location.href
    );

    if (clientTokenSet) {
      window.history.replaceState(
        null,
        "Redirection",
        authClient.redirectUris.login
      );

      authClient.storeClientTokenSet(clientTokenSet);
    }
  } catch (error) {
    log.warn(
      log.json`An error occured while trying to get the client token set ${error}`
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

  // All good
  return { type: "success" };
}
