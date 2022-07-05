import { derived } from "svelte/store";

import type { Common } from "@lgn/api/editor";
import type { NonEmptyArray } from "@lgn/web-client/src/lib/array";
import type { AsyncOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import { createAsyncStoreListOrchestrator } from "@lgn/web-client/src/orchestrators/async";

import { getActiveSceneIds } from "@/api";

import { allResources, allResourcesLoading } from "./allResources";

export type AllActiveScenesOrchestrator = AsyncOrchestrator<
  Common.ResourceDescription[]
>;

const allActiveScenesOrchestrator =
  createAsyncStoreListOrchestrator<Common.ResourceDescription[]>();

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

export type AllActiveScenesValue =
  | {
      rootScene: Common.ResourceDescription;
      scenes: NonEmptyArray<Common.ResourceDescription>;
    }[]
  | null;

export const allActiveScenes = derived(
  [allResources, allActiveSceneIdsOrchestrator.data],
  ([allResources, allActiveSceneIds]) => {
    if (!allResources || !allActiveSceneIds || !allActiveSceneIds.length) {
      return null;
    }

    return allActiveSceneIds.reduce((activeScenes, activeSceneId) => {
      const rootScene = allResources.find(
        (resource) => activeSceneId === resource.id
      );

      if (!rootScene) {
        return activeScenes;
      }

      return [
        ...activeScenes,
        {
          rootScene,
          scenes: allResources.filter((resource) =>
            resource.path.startsWith(rootScene.path)
          ) as NonEmptyArray<Common.ResourceDescription>,
        },
      ];
    }, [] as Exclude<AllActiveScenesValue, null>);
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
    return await allActiveSceneIdsOrchestrator.run(getActiveSceneIds);
  } catch (error) {
    allActiveScenesOrchestrator.error.set(error);
  }
}
