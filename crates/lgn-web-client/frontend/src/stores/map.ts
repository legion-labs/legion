import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

// TODO: Also create a map orchestrator, i.e. a map of store (as opposed to a "store of map")

export type MapValue<Key, Value> = Map<Key, Value>;

export type MapStore<Key, Value> = Writable<MapValue<Key, Value>> & {
  add(key: Key, value: Value): void;
  addAll(...values: [key: Key, value: Value][]): void;
  remove(key: Key): boolean;
  replace(key: Key, value: Value): void;
  updateAt(key: Key, f: (value: Value) => Value): void;
  empty(): void;
};

/**
 * Simple store that contains a map.
 */
export function createMapStore<Key, Value>(
  initialValue = new Map<Key, Value>()
): MapStore<Key, Value> {
  return {
    ...writable<Map<Key, Value>>(initialValue),

    /**
     * Adds a new value to the map if the provided key is not present yet.
     * Use `replace` if you want to fully replace a value located under the provided key.
     */
    add(key, value) {
      this.update((map) => (map.has(key) ? map : map.set(key, value)));
    },

    addAll(...values) {
      this.update((map) => {
        for (const [key, value] of values) {
          if (!map.has(key)) {
            map.set(key, value);
          }
        }

        return map;
      });
    },

    /**
     * Replaces a value located under the provided key, when the key is not present
     * in the map yet, it'll be added.
     */
    replace(key, value) {
      this.update((map) => map.set(key, value));
    },

    remove(key) {
      let removed = false;

      this.update((map) => {
        removed = map.delete(key);

        return map;
      });

      return removed;
    },

    updateAt(key, f) {
      this.update((map) =>
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        map.has(key) ? map.set(key, f(map.get(key)!)) : map
      );
    },

    empty() {
      this.set(new Map());
    },
  };
}
