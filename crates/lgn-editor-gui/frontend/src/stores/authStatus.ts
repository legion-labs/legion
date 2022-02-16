import { InitAuthStatus } from "@lgn/web-client/src/lib/auth";
import { Writable } from "@lgn/web-client/src/lib/store";

export default new Writable<InitAuthStatus | null>(null);
