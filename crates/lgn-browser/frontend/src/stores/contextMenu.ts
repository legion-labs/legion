import { Writable, writable } from "svelte/store";
import { Entry } from "../types/contextMenu";

export type Store<EntryRecord extends Record<string, unknown>> = {
  [Name in keyof EntryRecord]: {
    subscribe: Writable<
      Partial<Record<Name, Entry<EntryRecord[Name]>[]>>
    >["subscribe"];
    register<Name extends keyof EntryRecord>(
      name: Name,
      entries: Entry<EntryRecord[Name]>[]
    ): void;
  };
}[keyof EntryRecord];

export default function buildContextMenuStore<
  EntryRecord extends Record<string, unknown>
>(): Store<EntryRecord> {
  const { update, subscribe } = writable({});

  return {
    subscribe,
    /**
     * Register the context menu entries that can be used later on
     * by any dom element using the `contextMenu` action.
     */
    register(name, entries) {
      update((entrySets) => ({
        ...entrySets,
        [name]: entries,
      }));
    },
  };
}
