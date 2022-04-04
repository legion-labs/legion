import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
import type { AsyncOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import { createAsyncStoreListOrchestrator } from "@lgn/web-client/src/orchestrators/async";

import { getActiveScenes } from "@/api";

export type AllActiveScenesOrchestrator = AsyncOrchestrator<
  ResourceDescription[]
>;

const allActiveScenesOrchestrator =
  createAsyncStoreListOrchestrator<ResourceDescription[]>();

export const {
  data: allActiveScenes,
  error: allActiveScenesError,
  loading: allActiveScenesLoading,
} = allActiveScenesOrchestrator;

export async function fetchAllActiveScenes() {
  try {
    return allActiveScenesOrchestrator.run(getActiveScenes);
  } catch (error) {
    allActiveScenesOrchestrator.error.set(error);
  }
}

export async function initAllActiveScenesStream(pollInternal = 2_000) {
  await fetchAllActiveScenes();

  const intervalId = setInterval(() => {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    fetchAllActiveScenes();
  }, pollInternal);

  return () => clearInterval(intervalId);
}

export default allActiveScenesOrchestrator;
