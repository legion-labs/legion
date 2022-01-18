import { writable } from "svelte/store";

export type Entry =
  | {
      type: "item";
      label: string;
      tag?: "danger";
      onClick: (args: {
        close: () => void;
        payload?: unknown;
      }) => void | Promise<void>;
    }
  | { type: "separator" };

export type StoreValue<Name extends string = string> = Partial<
  Record<Name, Entry[]>
>;

export default function buildContextMenuStore<Name extends string = string>() {
  const { subscribe, update } = writable<StoreValue<Name>>({});

  return {
    subscribe,
    /**
     * Register the context menu entries that can be used later on
     * by any dom element using the `contextMenu` action.
     */
    register(name: Name, entries: Entry[]) {
      update((entriesRecord) => ({ ...entriesRecord, [name]: entries }));
    },
    /** Removes the context menu entries */
    remove(name: Name) {
      update(
        ({ [name]: _, ...entriesRecord }) => entriesRecord as StoreValue<Name>
      );
    },
  };
}
