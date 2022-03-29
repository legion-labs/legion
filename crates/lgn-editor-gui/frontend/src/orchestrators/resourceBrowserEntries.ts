import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";

import { createHierarchyTreeOrchestrator } from "./hierarchyTree";

export const resourceBrowserEntriesOrchestrator =
  createHierarchyTreeOrchestrator<ResourceDescription>();

export const {
  currentlyRenameEntry: currentlyRenameResourceEntry,
  currentEntry: currentResourceDescriptionEntry,
  entries: resourceEntries,
} = resourceBrowserEntriesOrchestrator;
