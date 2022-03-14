import type { Writable } from "svelte/store";
import { createAsyncStoreListOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";

export type AllResourcesValue = ResourceDescription[];

export type AllResourcesStore = Writable<AllResourcesValue>;

export default createAsyncStoreListOrchestrator<AllResourcesValue>();
