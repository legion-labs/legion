import { derived } from "svelte/store";

import { allResources } from "./allResources";
import { deriveHierarchyTreeOrchestrator } from "./hierarchyTree";

export const resourceBrowserEntriesOrchestrator =
  deriveHierarchyTreeOrchestrator(
    derived(allResources, (allResources) => allResources || [])
  );

export const {
  currentlyRenameEntry: currentlyRenameResourceEntry,
  currentEntry: currentResourceDescriptionEntry,
  entries: resourceEntries,
} = resourceBrowserEntriesOrchestrator;
