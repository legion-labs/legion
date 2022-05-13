import { writable } from "svelte/store";
import type { Writable } from "svelte/store";

export type AccessTokenValue = string | null;

export type AccessTokenStore = Writable<AccessTokenValue>;

export default writable<AccessTokenValue>(null);
