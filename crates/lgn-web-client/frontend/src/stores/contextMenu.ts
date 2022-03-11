import { writable } from "svelte/store";
import type { Entry } from "../types/contextMenu";

export function createContextMenuStore<Names extends string>() {
  return {
    ...writable<Partial<Record<Names, Entry[]>>>({}),

    register(name: Names, entries: Entry[]): void {
      this.update((entrySets) => ({
        ...entrySets,
        [name]: entries,
      }));
    },
  };
}
