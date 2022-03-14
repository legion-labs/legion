import type { Writable } from "svelte/store";
import { writable } from "svelte/store";
import type { InitAuthStatus } from "@lgn/web-client/src/lib/auth";

export type AuthStatusValue = InitAuthStatus | null;

export type AuthStatusStore = Writable<AuthStatusValue>;

export default writable<AuthStatusValue>(null);
