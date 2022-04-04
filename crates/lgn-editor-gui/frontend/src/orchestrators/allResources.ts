import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
import type { AsyncOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import { createAsyncStoreListOrchestrator } from "@lgn/web-client/src/orchestrators/async";

import { getAllResources } from "@/api";

export type AllResourcesOrchestrator = AsyncOrchestrator<ResourceDescription[]>;

const allResourcesOrchestrator =
  createAsyncStoreListOrchestrator<ResourceDescription[]>();

export const {
  data: allResources,
  error: allResourcesError,
  loading: allResourcesLoading,
} = allResourcesOrchestrator;

export function fetchAllResources(name?: string) {
  return allResourcesOrchestrator.run(() => getAllResources(name));
}

export default allResourcesOrchestrator;
