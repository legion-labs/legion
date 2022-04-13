/**
 * A store orchestrator that contains all the currently displayed items
 * (as a record of hierarchy tree Entries), the entry that's currently being renamed, etc...
 *
 * It could also contain an array of id pointing to the currently expanded entries in the tree.
 */
import type { Readable, Unsubscriber, Writable } from "svelte/store";
import { get, readable, writable } from "svelte/store";

import type { Entry, ItemBase } from "@/lib/hierarchyTree";
import { Entries } from "@/lib/hierarchyTree";

export type HierarchyTreeOrchestrator<Item extends ItemBase> = {
  currentlyRenameEntry: Writable<Entry<Item> | null>;
  currentEntry: Writable<Entry<Item> | null>;
  entries: Writable<Entries<Item>>;
  unsubscriber: Unsubscriber;
};

export function createHierarchyTreeOrchestrator<Item extends ItemBase>(
  items: Item[] = []
): HierarchyTreeOrchestrator<Item> {
  return deriveHierarchyTreeOrchestrator(readable(items));
}

export function deriveHierarchyTreeOrchestrator<Item extends ItemBase>(
  itemsStore: Readable<Item[]>
): HierarchyTreeOrchestrator<Item> {
  const entries = writable<Entries<Item>>(Entries.empty());

  const currentlyRenameEntry = writable<Entry<Item> | null>(null);

  const currentEntry = writable<Entry<Item> | null>(null);

  const unsubscriber = itemsStore.subscribe((items) => {
    const currentEntryValue = get(currentEntry);
    const currentRenameEntryValue = get(currentlyRenameEntry);

    // Reset current entry value to null if it's not present in the entries set anymore
    if (
      currentEntryValue &&
      !items.some((item) => currentEntryValue.item.id === item.id)
    ) {
      currentEntry.set(null);
    }

    // Reset currently rename entry value to null if it's not present in the entries set anymore
    if (
      currentRenameEntryValue &&
      !items.some((item) => currentRenameEntryValue.item.id === item.id)
    ) {
      currentlyRenameEntry.set(null);
    }

    entries.set(Entries.fromArray(items));
  });

  return { currentlyRenameEntry, currentEntry, entries, unsubscriber };
}
