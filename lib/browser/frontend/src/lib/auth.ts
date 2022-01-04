import { getCookie, setCookie } from "./cookie";

const authorizationUrl = new URL(
  "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=5m58nrjfv6kr144prif9jk62di&response_type=code&scope=aws.cognito.signin.user.admin+email+https://legionlabs.com/editor/allocate+openid+profile&redirect_uri=http://localhost:3000/&identity_provider=Azure"
);

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

export type ClientTokenSet = {
  access_token: string;
  refresh_token?: string;
  expires_in: number;
};

export type GetTokenSetRequest =
  | { type: "code"; code: string }
  | { type: "refreshToken"; refreshToken: string };

export interface Authenticator {
  getAuthorizationCodeInteractive(): void;

  getTokenSet(request: GetTokenSetRequest): Promise<ClientTokenSet>;
}

export class TokenCache<A extends Authenticator> {
  constructor(public authenticator: A) {}

  getAuthorizationCodeInteractive() {
    if (this.tokenIsInvalid()) {
      return this.authenticator.getAuthorizationCodeInteractive();
    }
  }

  getTokenSet(request: GetTokenSetRequest) {
    if (this.tokenIsInvalid()) {
      return this.authenticator.getTokenSet(request);
    }
  }

  tokenIsInvalid() {
    const expiresAt = getCookie("expires_at");
    const access_token = getCookie("access_token");

    return !expiresAt || !access_token || new Date(expiresAt) <= new Date();
  }
}

export class AwsCognitoClientAuthenticator implements Authenticator {
  private domainName: string;
  private region: string;
  private clientId: string;
  private scopes: string[];
  private identityProvider: string | null;
  private port: number;

  constructor(authorizationUrl: URL) {
    if (authorizationUrl.pathname != "/oauth2/authorize") {
      throw new Error("URL must be an AWS Cognito authorization URL");
    }

    const [domainName, auth, region, amazoncognito, com] =
      authorizationUrl.host.split(".");

    if (auth !== "auth" || amazoncognito !== "amazoncognito" || com !== "com") {
      throw new Error(
        "Host must respect the `<domain_name>.auth.<region>.amazoncognito.com` format"
      );
    }

    const clientId = authorizationUrl.searchParams.get("client_id");

    if (!clientId) {
      throw new Error("Client id not provided in URL search params");
    }

    const scopes =
      authorizationUrl.searchParams.get("scopes")?.split("+") || [];

    const identityProvider =
      authorizationUrl.searchParams.get("identity_provider");

    const redirectUri = authorizationUrl.searchParams.get("redirect_uri");

    if (!redirectUri) {
      throw new Error("Redirect URI not provided in URL search params");
    }

    const redirectUrl = new URL(redirectUri);

    const port = +redirectUrl.port || 80;

    this.clientId = clientId;
    this.domainName = domainName;
    this.port = port;
    this.region = region;
    this.scopes = scopes;
    this.identityProvider = identityProvider;
  }

  private baseUrl(path: string) {
    return new URL(
      `https://${this.domainName}.auth.${this.region}.amazoncognito.com/${path}`
    );
  }

  private get redirectUri() {
    return `http://localhost:${this.port}/`;
  }

  private get accessTokenUrl() {
    return this.baseUrl("oauth2/token");
  }

  private get userInfoUrl() {
    return this.baseUrl("oauth2/userInfo");
  }

  private get authorizationUrl() {
    const authorizationUrl = this.baseUrl("oauth2/authorize");

    authorizationUrl.searchParams.set("client_id", this.clientId);
    authorizationUrl.searchParams.set("response_type", "code");
    authorizationUrl.searchParams.set("scope", this.scopes.join("+"));
    authorizationUrl.searchParams.set("redirect_uri", this.redirectUri);

    if (this.identityProvider) {
      authorizationUrl.searchParams.set(
        "identity_provider",
        this.identityProvider
      );
    }

    return authorizationUrl;
  }

  getAuthorizationCodeInteractive() {
    window.location.href = this.authorizationUrl.toString();
  }

  async getTokenSet(request: GetTokenSetRequest): Promise<ClientTokenSet> {
    let body: URLSearchParams;

    switch (request.type) {
      case "code": {
        body = new URLSearchParams({
          grant_type: "authorization_code",
          client_id: this.clientId,
          code: request.code,
          redirect_uri: this.redirectUri,
        });

        break;
      }

      case "refreshToken": {
        body = new URLSearchParams({
          grant_type: "refresh_token",
          client_id: this.clientId,
          refresh_token: request.refreshToken,
          redirect_uri: this.redirectUri,
        });

        break;
      }

      default: {
        throw new Error(`Unexpected request: ${request}`);
      }
    }

    const requestInit: RequestInit = {
      method: "POST",
      mode: "cors",
      body,
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
    };

    const response = await fetch(
      new Request(this.accessTokenUrl.toString(), requestInit)
    );

    if (!response.ok) {
      throw new Error(await response.text());
    }

    return response.json();
  }

  async getUserInfo(accessToken: string): Promise<UserInfo> {
    const requestInit: RequestInit = {
      method: "GET",
      mode: "cors",
      headers: { Authorization: `Bearer ${accessToken}` },
    };

    const request = new Request(this.userInfoUrl.toString(), requestInit);

    const response = await fetch(request);

    if (!response.ok) {
      throw new Error(await response.text());
    }

    return response.json();
  }
}

export function createAwsCognito() {
  return new AwsCognitoClientAuthenticator(authorizationUrl);
}

export function createAwsCognitoTokenCache() {
  return new TokenCache(createAwsCognito());
}

/**
 * Takes a code inserted by AWS Cognito in the URL's query params,
 * set all cookies in the browser, and return the UserInfo.
 * @param code Code inserted by AWS Cognito in the URL's query params
 * @returns The user info
 */
export async function finalizeAwsCognitoAuth(
  awsCognitoTokenCache: TokenCache<AwsCognitoClientAuthenticator>,
  code: string
) {
  const clientTokenSet = await awsCognitoTokenCache.getTokenSet({
    type: "code",
    code,
  });

  if (!clientTokenSet) {
    return null;
  }

  const { access_token, expires_in, refresh_token } = clientTokenSet;

  const expiresAt = new Date(Date.now() + expires_in * 1000).toUTCString();

  setCookie("access_token", access_token, expires_in);

  if (refresh_token) {
    setCookie("refresh_token", refresh_token, expires_in);
  }

  setCookie("expires_at", expiresAt, expires_in);

  return awsCognitoTokenCache.authenticator.getUserInfo(access_token);
}

/** */
export async function scheduleRefreshClientTokenSet(
  awsCognitoTokenCache: TokenCache<AwsCognitoClientAuthenticator>
) {
  const expiresAtCookie = getCookie("expires_at");

  if (!expiresAtCookie) {
    return;
  }

  const expiresAt = +expiresAtCookie;

  if (isNaN(expiresAt)) {
    return;
  }

  const expiresIn = expiresAt - Date.now() - 10_000;

  const timeoutId = setTimeout(async () => {
    const refreshToken = getCookie("refresh_token");

    if (!refreshToken) {
      return;
    }

    await awsCognitoTokenCache.getTokenSet({
      type: "refreshToken",
      refreshToken,
    });
  }, expiresIn);

  return timeoutId;
}
