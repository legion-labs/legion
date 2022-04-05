import { derived } from "svelte/store";

import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
import type { AsyncOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import { createAsyncStoreListOrchestrator } from "@lgn/web-client/src/orchestrators/async";

import { getActiveSceneIds } from "@/api";

import { allResources, allResourcesLoading } from "./allResources";

export type AllActiveScenesOrchestrator = AsyncOrchestrator<
  ResourceDescription[]
>;

const allActiveScenesOrchestrator =
  createAsyncStoreListOrchestrator<ResourceDescription[]>();

// TODO: Uncomment this whole section when the scene explorer is more advanced

// export const {
//   data: allActiveScenes,
//   error: allActiveScenesError,
//   loading: allActiveScenesLoading,
// } = allActiveScenesOrchestrator;

// export async function fetchAllActiveScenes() {
//   try {
//     return allActiveScenesOrchestrator.run(getActiveScenes);
//   } catch (error) {
//     allActiveScenesOrchestrator.error.set(error);
//   }
// }

// export async function initAllActiveScenesStream(pollInternal = 2_000) {
//   await fetchAllActiveScenes();

//   const intervalId = setInterval(() => {
//     // eslint-disable-next-line @typescript-eslint/no-floating-promises
//     fetchAllActiveScenes();
//   }, pollInternal);

//   return () => clearInterval(intervalId);
// }

// export default allActiveScenesOrchestrator;

// TODO: Remove all the following code when the scene explorer is more advanced

const allActiveSceneIdsOrchestrator =
  createAsyncStoreListOrchestrator<string[]>();

export const allActiveScenes = derived(
  [allResources, allActiveSceneIdsOrchestrator.data],
  ([allResources, allActiveSceneIds]) => {
    if (!allResources || !allActiveSceneIds || !allActiveSceneIds.length) {
      return allResources;
    }

    return allResources.filter((resource) =>
      allActiveSceneIds.includes(resource.id)
    );
  }
);

export const allActiveScenesLoading = derived(
  [allResourcesLoading, allActiveSceneIdsOrchestrator.loading],
  ([allResourcesLoading, allActiveScenesLoading]) =>
    allResourcesLoading || allActiveScenesLoading
);

export const { error: allActiveScenesError } = allActiveSceneIdsOrchestrator;

export async function fetchAllActiveScenes() {
  try {
    return allActiveSceneIdsOrchestrator.run(getActiveSceneIds);
  } catch (error) {
    allActiveScenesOrchestrator.error.set(error);
  }
}
