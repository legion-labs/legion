import type { UserInfo } from "../lib/auth";
import type { AsyncOrchestrator } from "./async";
import { createAsyncStoreOrchestrator } from "./async";

export type UserInfoOrchestrator = AsyncOrchestrator<UserInfo>;

export default createAsyncStoreOrchestrator<UserInfo>();
