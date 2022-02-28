import { v4 as uuid } from "uuid";
import { components } from "./path";

export type Entry<Item> = {
  name: string;
  index: number | null;
  item: Item;
  subEntries: Entry<Item>[];
};

export function isEntry<Item>(
  entry: Entry<Item | symbol>
): entry is Entry<Item> {
  return typeof entry.item !== "symbol";
}

// TODO: Improve performance if needed, and stop using recursion
/** A wrapper class around the `Entry<Item>[]` type. */
export class Entries<Item extends { path: string }> {
  entries: Entry<Item | symbol>[];

  #size!: number;

  /**
   * Build an `Entries` object from any flat arrays of object.
   * Objects must contain a `path` attribute.
   *
   * ## Example
   *
   * ```typescript
   * const entries = Entries.fromArray([
   *   { path: "/foo/bar", value: "hello" },
   *   { path: "/foo/baz", value: "another hello" },
   *   { path: "/foo", value: "another value" },
   *   { path: "baz", value: "a baz value" },
   *   { path: "/foo/bar/baz", value: "another baz value" },
   * ]);
   *
   * const expectedEntries = {
   * };
   *
   * // Given an `assert` and a `deepEqual` function:
   * assert(deepEqual(entries.entries, expectedEntries));
   * ```
   */
  static fromArray<Item extends { path: string }>(
    items: Item[]
  ): Entries<Item> {
    if (!items.length) {
      return new Entries([]);
    }

    type Ref = {
      [key: string]: Ref;
    } & { subEntries: Entry<Item | symbol>[] };

    const entriesArray: Entry<Item>[] = [];
    const ref = { subEntries: entriesArray } as Ref;

    items.forEach((item) => {
      const pathComponents = components(item.path);

      pathComponents.reduce((ref, name, index) => {
        if (!ref[name]) {
          ref[name] = {
            subEntries: [] as Entry<Item | symbol>[],
          } as Ref;

          const entry = {
            name,
            // Null index, will be set again properly later
            index: null,
            // Svelte stringifies symbols when the're used as keys
            // here we get best of both worlds: our symbols are guaranteed
            // to be unique even when stringified
            item: index < pathComponents.length - 1 ? Symbol.for(uuid()) : item,
            subEntries: ref[name].subEntries,
          };

          if (ref.subEntries) {
            ref.subEntries.push(entry);
          } else {
            ref.subEntries = [entry];
          }
        }

        return ref[name];
      }, ref);
    });

    const entries = new Entries(entriesArray);

    entries.#sort();

    return entries;
  }

  /** Builds an `Entries` object from and array of `Entry` */
  constructor(entries: Entry<Item>[]) {
    this.entries = entries;

    this.recalculateSize();
  }

  /** Computes the size of the `Entries` */
  recalculateSize() {
    function count(entries: Entry<Item | symbol>[], size = 0): number {
      return entries.reduce((size, entry) => {
        const newSize = isEntry(entry) ? size + 1 : size;

        return entry.subEntries ? count(entry.subEntries, newSize) : newSize;
      }, size);
    }

    this.#size = count(this.entries);
  }

  #setIndices() {
    let index = 0;

    for (const entry of this) {
      entry.index = index++;
    }
  }

  // Could be exposed if needed
  #sort() {
    function sort(entries: Entry<Item | symbol>[]): void {
      entries
        .sort(function (entry1, entry2) {
          return entry1.name.localeCompare(entry2.name, undefined, {
            numeric: true,
            sensitivity: "base",
          });
        })
        .forEach((entry) => {
          if (entry.subEntries?.length) {
            sort(entry.subEntries);
          }
        });
    }

    sort(this.entries);

    this.#setIndices();
  }

  /**
   * Finds an entry in an `Entries` array.
   */
  find(pred: (entry: Entry<Item>) => boolean): Entry<Item> | null {
    for (const entry of this) {
      if (pred(entry)) {
        return entry;
      }
    }

    return null;
  }

  /**
   * Filters `Entries` based on a predicate.
   */
  filter(pred: (entry: Entry<Item | symbol>) => boolean): this {
    function filter(entries: Entry<Item | symbol>[]): Entry<Item | symbol>[] {
      return entries.reduce((acc, entry) => {
        if (!pred(entry)) {
          return acc;
        }

        if (entry.subEntries) {
          const subEntries = filter(entry.subEntries);

          return [
            ...acc,
            {
              ...entry,
              subEntries,
            },
          ];
        }

        return [...acc, entry];
      }, [] as Entry<Item | symbol>[]);
    }

    this.entries = filter(this.entries);

    this.recalculateSize();

    this.#setIndices();

    return this;
  }

  /**
   * Finds an entry index in an `Entries` array.
   *
   * Unlike `Array.prototype.findIndex`, this method returns `null` if the index is not found, not -1.
   */
  findIndex(pred: (entry: Entry<Item>) => boolean): number | null {
    for (const entry of this) {
      if (pred(entry)) {
        return entry.index;
      }
    }

    return null;
  }

  /**
   * Takes a whole `Entries` object and a function called for each entry in this object.
   *
   * If the function returns `null` nothing happens,
   * if an item and/or a name is returned, then the entry will be updated.
   */
  update(
    shouldUpdate: (
      entry: Entry<Item>
    ) => Pick<Entry<Item>, "item" | "name"> | null
  ): this {
    function update(entries: Entry<Item | symbol>[]): Entry<Item | symbol>[] {
      return entries.map((entry) => {
        if (isEntry(entry)) {
          const updatedEntry = shouldUpdate(entry);

          if (updatedEntry) {
            return {
              ...entry,
              ...updatedEntry,
              name:
                ("name" in updatedEntry && updatedEntry.name.trim()) ||
                entry.name,
            };
          }
        }

        if (!entry.subEntries) {
          return entry;
        }

        return { ...entry, subEntries: update(entry.subEntries) };
      });
    }

    this.entries = update(this.entries);

    this.#sort();

    return this;
  }

  insert(item: Item): this {
    function insert(
      [part, ...parts]: string[],
      entries: Entry<Item | symbol>[],
      item: Item
    ): Entry<Item | symbol>[] {
      if (!parts.length) {
        const newEntry: Entry<Item> = {
          index: null,
          item,
          name: part,
          subEntries: [],
        };

        return [...entries, newEntry];
      }

      const entry = entries.find((entry) => entry.name === part);

      if (!entry) {
        return entries;
      }

      entry.subEntries = insert(parts, entry.subEntries || [], item);

      return entries;
    }

    this.entries = insert(components(item.path), this.entries, item);

    this.#sort();

    this.recalculateSize();

    return this;
  }

  /** Get an entry from its index */
  getFromIndex(index: number): Entry<Item> | null {
    for (const entry of this) {
      if (entry.index === index) {
        return entry;
      }
    }

    return null;
  }

  remove(removedEntry: Entry<Item>): this {
    return this.filter((entry) => entry !== removedEntry);
  }

  isEmpty() {
    return this.#size === 0;
  }

  intoItems(): Item[] {
    const items = [];

    for (const entry of this) {
      items.push(entry.item);
    }

    return items;
  }

  size() {
    return this.#size;
  }

  [Symbol.iterator]() {
    function* iter(entries: Entry<Item | symbol>[]): Generator<Entry<Item>> {
      for (const entry of entries) {
        if (isEntry(entry)) {
          yield entry;
        }

        if (entry.subEntries) {
          yield* iter(entry.subEntries);
        }
      }
    }

    return iter(this.entries);
  }
}
