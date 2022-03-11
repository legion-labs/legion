import { createAsyncStoreListOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";

export default createAsyncStoreListOrchestrator<ResourceDescription[]>();
