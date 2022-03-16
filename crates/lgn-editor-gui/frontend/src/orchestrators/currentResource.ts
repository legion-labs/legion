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

export function fetchCurrentResourceDescription(
  currentResourceDescription: ResourceDescription
) {
  // Ignore folder without id
  if (!currentResourceDescription.id) {
    return;
  }

  try {
    currentResourceOrchestrator.run(() => {
      if (!currentResourceDescription) {
        throw new Error("Current resource description not found");
      }

      updateSelection(currentResourceDescription.id);

      return getResourceProperties(currentResourceDescription);
    });
  } catch (error) {
    notifications.push(Symbol(), {
      type: "error",
      title: "Resources",
      message: "An error occured while loading the resource",
    });

    log.error(
      log.json`An error occured while loading the resource ${currentResourceDescription}: ${error}`
    );
  }
}

export default currentResourceOrchestrator;
