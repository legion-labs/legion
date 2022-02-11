import { components } from "./path";

export type Entry<Item> = {
  name: string;
  index: number;
  item: Item;
  subEntries: Entry<Item>[] | null;
  // eslint-disable-next-line @typescript-eslint/ban-types
};

// TODO: Improve performance if needed, and stop using recursion
/** A wrapper class around the `Entry<Item>[]` type. */
export class Entries<Item> {
  entries: Entry<Item>[];

  #size: number | null = null;

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
  static fromArray<PItem extends { path: string }, AltItem>(
    items: PItem[],
    /** This function is called when an `Item` is not present, typically used for "folders" */
    buildItemFromName: (name: string) => PItem | AltItem
  ): Entries<PItem | AltItem> {
    if (!items.length) {
      return new Entries([]);
    }

    type Ref = {
      [key: string]: Ref;
    } & { subEntries: Entry<PItem | AltItem>[] | null };

    const entriesArray: Entry<PItem>[] = [];
    const ref = { subEntries: entriesArray } as Ref;

    items.forEach((item) => {
      const pathComponents = components(item.path);

      pathComponents.reduce((ref, name, index) => {
        if (!ref[name]) {
          ref[name] = {
            subEntries: index === pathComponents.length - 1 ? null : [],
          } as Ref;

          const entry = {
            name,
            // Dumb index, will be set again properly later
            index: -1,
            item:
              index < pathComponents.length - 1
                ? buildItemFromName(name)
                : item,
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
  }

  /** Computes the size of the `Entries` */
  get size(): number {
    if (typeof this.#size === "number") {
      return this.#size;
    }

    function count(entries: Entry<Item>[], size = 0): number {
      return entries.reduce(
        (size, entry) =>
          entry.subEntries ? count(entry.subEntries, size + 1) : size + 1,
        size
      );
    }

    this.#size = count(this.entries);

    return this.#size;
  }

  #setIndices() {
    for (const [index, entry] of this) {
      entry.index = index;
    }
  }

  // Could be exposed if needed
  #sort() {
    function sort(entries: Entry<Item>[]): void {
      entries
        .sort((entry1, entry2) => (entry1.name > entry2.name ? 1 : -1))
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
    for (const [, entry] of this) {
      if (pred(entry)) {
        return entry;
      }
    }

    return null;
  }

  /**
   * Filters `Entries` based on a predicate.
   */
  filter(pred: (entry: Entry<Item>) => boolean): this {
    function filter(entries: Entry<Item>[]): Entry<Item>[] {
      return entries.reduce((acc, entry) => {
        if (!pred(entry)) {
          return acc;
        }

        if (entry.subEntries) {
          return [...acc, { ...entry, subEntries: filter(entry.subEntries) }];
        }

        return [...acc, entry];
      }, [] as Entry<Item>[]);
    }

    this.entries = filter(this.entries);

    // Resets the size so it's computed again if needed
    this.#size = null;

    this.#setIndices();

    return this;
  }

  /**
   * Finds an entry index in an `Entries` array.
   *
   * Unlike `Array.prototype.findIndex`, this method returns `null` if the index is not found, not -1.
   */
  findIndex(pred: (entry: Entry<Item>) => boolean): number | null {
    for (const [index, entry] of this) {
      if (pred(entry)) {
        return index;
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
    function update(entries: Entry<Item>[]): Entry<Item>[] {
      return entries.map((entry) => {
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

        if (!entry.subEntries) {
          return entry;
        }

        return {
          ...entry,
          subEntries: update(entry.subEntries),
        };
      });
    }

    this.entries = update(this.entries);

    this.#sort();

    return this;
  }

  insert<PItem extends { path: string }>(item: PItem): this {
    function insert(
      [part, ...parts]: string[],
      entries: Entry<Item>[]
    ): Entry<Item>[] {
      if (!parts.length) {
        const newEntry: Entry<Item> = {
          index: -1,
          item: item as unknown as Item,
          name: part,
          subEntries: null,
        };

        return [...entries, newEntry];
      }

      const entry = entries.find((entry) => entry.name === part);

      if (!entry) {
        return entries;
      }

      entry.subEntries = insert(parts, entry.subEntries || []);

      return entries;
    }

    this.entries = components(item.path).reduce((acc, part) => {
      acc;

      return acc;
    }, this.entries);

    this.entries = insert(components(item.path), this.entries);

    this.#sort();

    this.#size = null;

    return this;
  }

  /** Get an entry from its index */
  getFromIndex(index: number): Entry<Item> | null {
    for (const [entryIndex, entry] of this) {
      if (entryIndex === index) {
        return entry;
      }
    }

    return null;
  }

  remove(removedEntry: Entry<Item>): this {
    return this.filter((entry) => entry !== removedEntry);
  }

  [Symbol.iterator]() {
    let index = 0;

    function* iter(
      entries: Entry<Item>[]
    ): Generator<[index: number, entry: Entry<Item>]> {
      for (const entry of entries) {
        yield [index++, entry];

        if (entry.subEntries) {
          yield* iter(entry.subEntries);
        }
      }
    }

    return iter(this.entries);
  }
}
