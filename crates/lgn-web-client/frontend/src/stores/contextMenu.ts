import type { Writable } from "svelte/store";
import { writable } from "svelte/store";
import type { Entry } from "../types/contextMenu";

export type ContextMenuValue<Names extends string> = Partial<
  Record<Names, Entry[]>
>;

export type ContextMenuStore<Names extends string> = Writable<
  ContextMenuValue<Names>
> & {
  register(name: Names, entries: Entry[]): void;
};

export function createContextMenuStore<
  Names extends string
>(): ContextMenuStore<Names> {
  return {
    ...writable({}),

    register(name, entries) {
      this.update((entrySets) => ({
        ...entrySets,
        [name]: entries,
      }));
    },
  };
}
