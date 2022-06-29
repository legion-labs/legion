import type { Common } from "@lgn/api/editor";
import type { AsyncOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import { createAsyncStoreListOrchestrator } from "@lgn/web-client/src/orchestrators/async";

import { getAllResources } from "@/api";
import { fetchStagedResources } from "@/stores/stagedResources";

export type AllResourcesOrchestrator = AsyncOrchestrator<
  Common.ResourceDescription[]
>;

const allResourcesOrchestrator =
  createAsyncStoreListOrchestrator<Common.ResourceDescription[]>();

export const {
  data: allResources,
  error: allResourcesError,
  loading: allResourcesLoading,
} = allResourcesOrchestrator;

export async function fetchAllResources(name?: string) {
  const allResources = allResourcesOrchestrator.run(() =>
    getAllResources(name)
  );

  await fetchStagedResources();

  return allResources;
}

export default allResourcesOrchestrator;
