import { derived } from "svelte/store";

import { allActiveScenes } from "./allActiveScenes";
import { deriveHierarchyTreeOrchestrator } from "./hierarchyTree";

export const sceneExplorerEntriesOrchestrator = deriveHierarchyTreeOrchestrator(
  derived(allActiveScenes, (allActiveScenes) => allActiveScenes || [])
);

export const {
  currentlyRenameEntry: currentlyRenameSceneEntry,
  currentEntry: currentSceneDescriptionEntry,
  entries: sceneEntries,
} = sceneExplorerEntriesOrchestrator;
