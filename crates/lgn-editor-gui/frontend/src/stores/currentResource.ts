import { ResourceWithProperties } from "@/lib/propertyGrid";
import { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
import { AsyncStoreOrchestrator } from "@lgn/web-client/src/stores/asyncStore";
import notifications from "@/stores/notifications";
import { getResourceProperties, updateSelection } from "@/api";
import log from "@lgn/web-client/src/lib/log";

const currentResourceStore =
  new AsyncStoreOrchestrator<ResourceWithProperties>();

export function fetchCurrentResourceDescription(
  currentResourceDescription: ResourceDescription
) {
  // Ignore folder without id
  if (!currentResourceDescription.id) {
    return;
  }

  try {
    currentResourceStore.run(() => {
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

export default currentResourceStore;
