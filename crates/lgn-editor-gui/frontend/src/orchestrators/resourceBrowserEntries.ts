import { derived } from "svelte/store";

import { allResources } from "./allResources";
import { deriveHierarchyTreeOrchestrator } from "./hierarchyTree";

// TODO: Clean the subscription by calling the returned `unsubscriber` method
export const resourceBrowserEntriesOrchestrator =
  deriveHierarchyTreeOrchestrator(
    derived(allResources, (allResources) => allResources || [])
  );

export const {
  currentlyRenameEntry: currentlyRenameResourceEntry,
  currentEntry: currentResourceDescriptionEntry,
  entries: resourceEntries,
} = resourceBrowserEntriesOrchestrator;
