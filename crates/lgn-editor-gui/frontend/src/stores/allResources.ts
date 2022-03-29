import type { Writable } from "svelte/store";

import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
import { createAsyncStoreListOrchestrator } from "@lgn/web-client/src/orchestrators/async";

export type AllResourcesValue = ResourceDescription[];

export type AllResourcesStore = Writable<AllResourcesValue>;

export default createAsyncStoreListOrchestrator<AllResourcesValue>();
