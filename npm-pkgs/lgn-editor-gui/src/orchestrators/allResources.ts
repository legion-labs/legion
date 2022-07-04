import type { Common } from "@lgn/api/editor";
import { displayError } from "@lgn/web-client/src/lib/errors";
import log from "@lgn/web-client/src/lib/log";
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

  try {
    await fetchStagedResources();
  } catch (error) {
    log.error("staged-resources", displayError(error));
  }

  return allResources;
}

export default allResourcesOrchestrator;
