import type { ResourceWithProperties } from "@/lib/propertyGrid";
import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
import type { AsyncOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import { createAsyncStoreOrchestrator } from "@lgn/web-client/src/orchestrators/async";
import notifications from "@/stores/notifications";
import { getResourceProperties, updateSelection } from "@/api";
import log from "@lgn/web-client/src/lib/log";

export type CurrentResourceOrchestrator =
  AsyncOrchestrator<ResourceWithProperties>;

const currentResourceOrchestrator: CurrentResourceOrchestrator =
  createAsyncStoreOrchestrator();

export const { data: currentResource, error: currentResourceError } =
  currentResourceOrchestrator;

export function fetchCurrentResourceDescription(
  id: string,
  { notifySelection = true }: { notifySelection?: boolean } = {}
) {
  // Ignore folder without id
  if (!id) {
    return;
  }

  try {
    currentResourceOrchestrator.run(() => {
      if (notifySelection) {
        updateSelection(id);
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
