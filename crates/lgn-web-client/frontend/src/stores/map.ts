import { Writable } from "../lib/store";

// TODO: Also create a map orchestrator, i.e. a map of store (as opposed to a "store of map")

/**
 * Simple store that contains a map.
 */
export default class<Value> extends Writable<Map<symbol, Value>> {
  constructor(initialValue = new Map<symbol, Value>()) {
    super(initialValue);
  }

  /**
   * Adds a new value to the map if the provided key is not present yet.
   * Use `replace` if you want to fully replace a value located under the provided key.
   */
  add(key: symbol, value: Value) {
    this.update((map) => (map.has(key) ? map : map.set(key, value)));
  }

  addAll(...values: [key: symbol, value: Value][]) {
    this.update((map) => {
      for (const [key, value] of values) {
        if (!map.has(key)) {
          map.set(key, value);
        }
      }

      return map;
    });
  }

  /**
   * Replaces a value located under the provided key, when the key is not present
   * in the map yet, it'll be added.
   */
  replace(key: symbol, value: Value) {
    this.update((map) => map.set(key, value));
  }

  remove(key: symbol): boolean {
    let removed = false;

    this.update((map) => {
      removed = map.delete(key);

      return map;
    });

    return removed;
  }

  updateAt(key: symbol, f: (value: Value) => Value) {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    this.update((map) => (map.has(key) ? map.set(key, f(map.get(key)!)) : map));
  }

  empty() {
    this.set(new Map());
  }
}
