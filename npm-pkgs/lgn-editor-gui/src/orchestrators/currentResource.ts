import { derived } from "svelte/store";

import log from "@lgn/web-client/src/lib/log";
import type { AsyncOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import { createAsyncStoreOrchestrator } from "@lgn/web-client/src/orchestrators/async";

import { getResourceProperties, updateSelection } from "@/api";
import { fileName } from "@/lib/path";
import type { ResourceWithProperties } from "@/lib/propertyGrid";
import notifications from "@/stores/notifications";

export type CurrentResourceOrchestrator =
  AsyncOrchestrator<ResourceWithProperties>;

const currentResourceOrchestrator: CurrentResourceOrchestrator =
  createAsyncStoreOrchestrator();

export const { data: currentResource, error: currentResourceError } =
  currentResourceOrchestrator;

export const currentResourceName = derived(currentResource, (currentResource) =>
  currentResource ? fileName(currentResource.description.path) : null
);

export async function fetchCurrentResourceDescription(
  id: string,
  { notifySelection = true }: { notifySelection?: boolean } = {}
): Promise<void> {
  // Ignore folder without id
  if (!id) {
    return;
  }

  try {
    await currentResourceOrchestrator.run(async () => {
      if (notifySelection) {
        await updateSelection(id);
      }

      return getResourceProperties(id);
    });
  } catch (error) {
    notifications.push(Symbol(), {
      type: "error",
      title: "Resources",
      message: "An error occured while loading the resource",
    });

    log.error(
      log.json`An error occured while loading the resource ${id}: ${error}`
    );
  }
}

export default currentResourceOrchestrator;
