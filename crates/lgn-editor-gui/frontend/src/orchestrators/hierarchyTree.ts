/**
 * A store orchestrator that contains all the currently displayed items
 * (as a record of hierarchy tree Entries), the entry that's currently being renamed, etc...
 *
 * It could also contain an array of id pointing to the currently expanded entries in the tree.
 */

import type { Writable } from "svelte/store";
import { writable } from "svelte/store";
import type { Entry } from "@/lib/hierarchyTree";
import { Entries } from "@/lib/hierarchyTree";

export type HierarchyTreeOrchestrator<Item extends { path: string }> = {
  currentlyRenameEntry: Writable<Entry<Item> | null>;
  entries: Writable<Entries<Item>>;

  load(resources: Item[]): void;
};

export function createHierarchyTreeOrchestrator<Item extends { path: string }>(
  resources: Item[] = []
): HierarchyTreeOrchestrator<Item> {
  return {
    currentlyRenameEntry: writable<Entry<Item> | null>(null),

    entries: writable<Entries<Item>>(Entries.fromArray(resources)),

    /** Loads an array of element as hierarchy tree entries in the store */
    load(resources) {
      this.entries.set(Entries.fromArray(resources));
    },
  };
}
