/**
 * A store orchestrator that contains all the currently displayed items
 * (as a record of hierarchy tree Entries), the entry that's currently being renamed, etc...
 *
 * It could also contain an array of id pointing to the currently expanded entries in the tree.
 */

import { Entries, Entry } from "@/lib/hierarchyTree";
import { Orchestrator, Writable } from "@lgn/web-client/src/lib/store";

interface IconSetter<Item> {
  (entry: Entry<Item | symbol>): void;
}

export default class<Item extends { path: string }> implements Orchestrator {
  name = "hierarchTree";

  // `symbol` is used only for folder like resources
  // (which will be replaced by proper resources eventually)

  currentlyRenameEntry: Writable<Entry<Item | symbol> | null>;
  entries: Writable<Entries<Item | symbol>>;

  constructor(resources: Item[] = []) {
    this.currentlyRenameEntry = new Writable<Entry<Item | symbol> | null>(null);

    this.entries = new Writable(Entries.fromArray(resources));
  }

  /** Loads an array of element as hierarchy tree entries in the store */
  load(resources: Item[]) {
    const entries = Entries.fromArray(resources);

    this.entries.set(entries);
  }
}
