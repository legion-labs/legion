import { writable } from "svelte/store";
import type { InitAuthStatus } from "@lgn/web-client/src/lib/auth";

export default writable<InitAuthStatus | null>(null);
