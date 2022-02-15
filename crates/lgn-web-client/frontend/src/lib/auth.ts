import userInfo from "../stores/userInfo";
import { getCookie, setCookie } from "./cookie";
import log from "./log";
import getPkce from "oauth-pkce";
import { invoke } from "@tauri-apps/api";

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

class Client {
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
    redirectUri,
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
    if (redirectUri || this.config.redirectUri) {
      authorizationUrl.searchParams.set(
        "redirect_uri",
        (redirectUri || this.config.redirectUri) as string
      );
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

class LegionClient extends Client {
  #loginConfig: LoginConfig;

  constructor(
    issuerConfiguration: IssuerConfiguration,
    clientId: string,
    config: { redirectUri?: string },
    loginConfig: LoginConfig
  ) {
    super(issuerConfiguration, clientId, config);

    this.#loginConfig = loginConfig;
  }

  get accessToken() {
    const cookieName = this.#loginConfig.cookies?.accessToken || "access_token";

    return getCookie(cookieName);
  }

  async login() {
    if (window.__TAURI__) {
      try {
        await userInfo.run(async () => {
          const userInfo = (await invoke(
            "plugin:browser|authenticate"
          )) as UserInfo;

          log.debug("auth", userInfo);

          return userInfo;
        });
      } catch {
        // Nothing we can do about this but warn the user
        log.error("Couldn't authenticate the user");
      }
    }

    if (this.accessToken) {
      return;
    }

    const { challenge, verifier } = await PkceChallenge.newRandomSha256();

    const authorizeUrl = authClient.authorizeUrl({
      responseType: "code",
      scopes: this.#loginConfig.scopes,
      extraParams: this.#loginConfig.extraParams,
      pkceChallenge: challenge,
    });

    const popupWindow = window.open(
      authorizeUrl.toString(),
      this.#loginConfig.popupTitle,
      `height=600px, width=600px, status=yes, toolbar=no, menubar=no, location=no, top=${
        window.innerHeight / 2 - /* config.height */ 600 / 2 + window.screenTop
      }, left=${
        window.innerWidth / 2 - /* config.width */ 600 / 2 + window.screenLeft
      }`
    );

    if (!popupWindow) {
      throw new Error("Couldn't open auth popup");
    }

    popupWindow.focus();

    const code = await new Promise<string>((resolve, reject) => {
      const intervalId = setInterval(() => {
        try {
          const redirectUri =
            this.#loginConfig.redirectUri || this.config.redirectUri;

          if (!redirectUri) {
            throw new Error("No redirect uri specified");
          }

          if (popupWindow.location.origin === new URL(redirectUri).origin) {
            clearInterval(intervalId);

            console.log(popupWindow.location);

            const searchParams = new URLSearchParams(
              popupWindow.location.search
            );

            const code = searchParams.get("code");

            if (!code) {
              throw new Error("Code search param not found in url");
            }

            popupWindow.close();

            resolve(code);
          }
        } catch (error) {
          clearInterval(intervalId);

          reject(error);
        }
      }, 100);
    });

    const clientTokenSet = await authClient.exchangeCode(code, {
      pkceVerifier: verifier,
    });

    if (!clientTokenSet) {
      return null;
    }

    const { access_token, expires_in, refresh_token } = clientTokenSet;

    setCookie(
      this.#loginConfig.cookies?.accessToken || "access_token",
      access_token,
      expires_in
    );

    if (refresh_token) {
      setCookie(
        this.#loginConfig.cookies?.refreshToken || "refresh_token",
        refresh_token,
        expires_in
      );
    }

    await userInfo.run(() => this.userInfo());
  }

  override async userInfo(): Promise<UserInfo> {
    const accessToken = window.__TAURI__
      ? await invoke("plugin:browser|get_access_token")
      : getCookie(this.#loginConfig.cookies?.accessToken || "access_token");

    if (!accessToken) {
      throw new Error("Access token not found");
    }

    return super.userInfo(accessToken);
  }
}

export let authClient: LegionClient;

export async function initAuthClient({
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
}) {
  if (authClient && !force) {
    return;
  }

  const issuerConfiguration = await IssuerConfiguration.fromString(issuerUrl);

  const client = new LegionClient(
    issuerConfiguration,
    clientId,
    { redirectUri },
    loginConfig
  );

  authClient = client;
}

/**
 * If the `forceAuth` option is `true` the unauthenticated users
 * will have to log in.
 */
export async function initAuth({ forceAuth }: { forceAuth: boolean }) {
  try {
    await userInfo.run(() => authClient.userInfo());
  } catch {
    if (forceAuth) {
      await authClient.login();
    }
  }
}
