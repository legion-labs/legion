import type { UserInfo } from "../lib/auth";
import { createAsyncStoreOrchestrator } from "../orchestrators/async";

export default createAsyncStoreOrchestrator<UserInfo>();
