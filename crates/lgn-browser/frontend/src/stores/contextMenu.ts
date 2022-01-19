import { get, Writable, writable } from "svelte/store";
import log from "../lib/log";

export type Entry<Payload> =
  | {
      type: "item";
      label: string;
      tag?: "danger";
      onClick: (args: {
        close: () => void;
        payload: Payload;
      }) => void | Promise<void>;
    }
  | { type: "separator" };

export type Store<EntryRecord extends Record<string, unknown>> = {
  entryRecord: Writable<
    Partial<Record<keyof EntryRecord, Entry<EntryRecord[keyof EntryRecord]>[]>>
  >;
  activeEntrySet: Writable<{
    name: keyof EntryRecord;
    payload: EntryRecord[keyof EntryRecord];
  } | null>;
  setActiveEntrySet<Name extends keyof EntryRecord>(
    name: Name,
    payload: EntryRecord[Name]
  ): void;
  removeActiveEntrySet(): void;
  register<Name extends keyof EntryRecord>(
    name: Name,
    entries: Entry<EntryRecord[Name]>[]
  ): void;
  remove<Name extends keyof EntryRecord>(name: Name): void;
};

export default function buildContextMenuStore<
  EntryRecord extends Record<string, unknown>
>(): Store<EntryRecord> {
  const entryRecordStore: Store<EntryRecord>["entryRecord"] = writable({});

  const activeEntrySetStore: Store<EntryRecord>["activeEntrySet"] =
    writable(null);

  return {
    entryRecord: entryRecordStore,
    activeEntrySet: activeEntrySetStore,
    /**
     * "Activates" a context menu entry set
     * When the menu will be opened, it'll show the entries for this entry set
     */
    setActiveEntrySet(name, payload) {
      const entrySets = get(entryRecordStore);

      if (name in entrySets) {
        return activeEntrySetStore.set({ name, payload });
      }

      log.warn(
        `Trying to activate a context menu entry set that has not been registered: ${name}`
      );

      activeEntrySetStore.set(null);
    },
    /**
     * Sets the active entry set to "null"
     */
    removeActiveEntrySet() {
      activeEntrySetStore.set(null);
    },
    /**
     * Register the context menu entries that can be used later on
     * by any dom element using the `contextMenu` action.
     */
    register(name, entries) {
      entryRecordStore.update((entrySets) => ({
        ...entrySets,
        [name]: entries,
      }));
    },
    /** Removes the context menu entries */
    remove(name) {
      entryRecordStore.update(
        ({ [name]: _, ...entrySets }) =>
          entrySets as Partial<
            Record<keyof EntryRecord, Entry<EntryRecord[keyof EntryRecord]>[]>
          >
      );
    },
  };
}
