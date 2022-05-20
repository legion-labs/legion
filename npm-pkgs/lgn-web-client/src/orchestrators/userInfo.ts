import type { UserInfo } from "../lib/auth";
import type { AsyncOrchestrator } from "./async";
import { createAsyncStoreOrchestrator } from "./async";

export type UserInfoOrchestrator = AsyncOrchestrator<UserInfo>;

const userInfoStore = createAsyncStoreOrchestrator<UserInfo>();

export const {
  data: userInfo,
  error: userInfoError,
  loading: userInfoLoading,
} = userInfoStore;

export default userInfoStore;
