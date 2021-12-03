/* tslint:disable */
/* eslint-disable */
/**
* If the access token (stored in cookies) is not found or expired
* the user is redirected to Cognito in order to issue a new token.
*
* Ultimately, after the user is authenticated, they will be redirected
* to the aplication with a code, provided by Cognito, in the URL.
*/
export function getAuthorizationCodeInteractive(): void;
/**
* If the access token (stored in cookies) is not found or expired
* then a `ClientTokenSet` will be fetched and cookies will be set accordingly
* in the browser.
*
* At that point, the user is authenticated.
* @param {string} code
* @returns {Promise<void>}
*/
export function finalizeAwsCognitoAuth(code: string): Promise<void>;
/**
* Gets the access token currently stored in cookies
* @returns {string | undefined}
*/
export function getAccessToken(): string | undefined;
/**
* Use the provided access token to fetch the authed user info.
* @param {string} access_token
* @returns {Promise<any>}
*/
export function getUserInfo(access_token: string): Promise<any>;

export type UserInfo = Readonly<{
    sub: string,
    name: string | null,
    given_name: string | null,
    family_name: string | null,
    middle_name: string | null,
    nickname: string | null,
    preferred_username: string | null,
    profile: string | null,
    picture: string | null,
    website: string | null,
    email: string | null,
    email_verified: boolean,
    gender: string | null,
    birthdate: string | null,
    zoneinfo: string | null,
    locale: string | null,
    phone_number: string | null,
    phone_number_verified: boolean,
    updated_at: string | null,

    // Azure-specific fields.
    //
    // This is a merely a convention, but we need one.
    //
    // These fields contains the Azure-specific information about the user, which allow us to query
    // the Azure API for extended user information (like the user's photo).
    "custom:azure_oid": string | null,
    "custom:azure_tid": string | null,
}>;



export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly getAuthorizationCodeInteractive: () => void;
  readonly finalizeAwsCognitoAuth: (a: number, b: number) => number;
  readonly getAccessToken: (a: number) => void;
  readonly getUserInfo: (a: number, b: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2df121f1e0117d88: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h1935289b1a5a7a2b: (a: number, b: number, c: number, d: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
