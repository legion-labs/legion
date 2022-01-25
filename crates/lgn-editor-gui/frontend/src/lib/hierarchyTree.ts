import { prependIfNonPresent } from "./array";
import { components } from "./path";

export type Entry<Item, WithIndex extends boolean = true> = {
  name: string;
  depth: number;
  item: Item;
  subEntries: Entry<Item, WithIndex>[] | null;
  // eslint-disable-next-line @typescript-eslint/ban-types
} & (WithIndex extends true ? { index: number } : {});

/** A wrapper class around the `Entry<Item>[]` type. */
export class Entries<Item> {
  entries: Entry<Item>[];

  #size: number | null = null;

  // TODO: Improve performance if needed
  /**
   * Build an `Entries` object from any flat arrays of object.
   * Objects must contain a `path` attribute.
   *
   * ## Example
   *
   * ```typescript
   * const entries = Entries.unflatten([
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
  static unflatten<Item extends { path: string }, AltItem>(
    items: Item[],
    /** This function is called when an `Item` is not present, typically used for "folders" */
    buildItemFromName: (name: string) => Item | AltItem
  ): Entries<Item | AltItem> {
    if (!items.length) {
      return new Entries([]);
    }

    function buildEntriesFromPathComponents(
      [pathComponent, ...otherPathComponents]: string[],
      item: Item,
      entries: Entry<Item | AltItem, false>[],
      depth = 0
    ): Entry<Item | AltItem, false>[] {
      const extendedEntries = prependIfNonPresent(
        entries,
        (entry) => entry.name === pathComponent,
        () => ({
          depth,
          name: pathComponent,
          item: otherPathComponents.length
            ? buildItemFromName(pathComponent)
            : item,
          subEntries: null,
        })
      );

      return extendedEntries.map((entry) => {
        if (entry.name !== pathComponent) {
          return entry;
        }

        return {
          ...entry,
          subEntries: otherPathComponents.length
            ? buildEntriesFromPathComponents(
                otherPathComponents,
                item,
                entry.subEntries || [],
                depth + 1
              )
            : entry.subEntries || null,
        };
      });
    }

    function buildEntriesFromItems(
      [item, ...otherItems]: Item[],
      entries: Entry<Item | AltItem, false>[] = []
    ): Entry<Item | AltItem, false>[] {
      const pathComponents = components(item.path);

      if (!pathComponents.length) {
        return entries;
      }

      const populatedEntries = buildEntriesFromPathComponents(
        pathComponents,
        item,
        entries
      );

      return otherItems.length
        ? buildEntriesFromItems(otherItems, populatedEntries)
        : populatedEntries;
    }

    const entries = buildEntriesFromItems(items);

    if (!entries.length) {
      return new Entries([]);
    }

    function sort(entries: Entry<Item | AltItem, false>[]): void {
      entries
        .sort((entry1, entry2) => (entry1.name > entry2.name ? 1 : -1))
        .map((entry) => ({
          ...entry,
          subEntries: entry.subEntries ? sort(entry.subEntries) : null,
        }));
    }

    sort(entries);

    let index = 0;

    function addIndex(
      entries: Entry<Item | AltItem, false>[]
    ): Entry<Item | AltItem>[] {
      return entries.map((entry) => ({
        ...entry,
        index: index++,
        subEntries: entry.subEntries ? addIndex(entry.subEntries) : null,
      }));
    }

    return new Entries(addIndex(entries));
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
        (s, entry) =>
          entry.subEntries ? count(entry.subEntries, s + 1) : s + 1,
        size
      );
    }

    this.#size = count(this.entries);

    return this.#size;
  }

  // TODO: Improve performance if needed
  /**
   * Finds an entry in an `Entries` array.
   */
  find(pred: (entry: Entry<Item>) => boolean): Entry<Item> | null {
    function find(entries: Entry<Item>[]): Entry<Item> | null {
      for (const entry of entries) {
        if (pred(entry)) {
          return entry;
        }

        if (entry.subEntries) {
          const foundEntry = find(entry.subEntries);

          if (foundEntry) {
            return foundEntry;
          }
        }
      }

      return null;
    }

    return find(this.entries);
  }

  // TODO: Improve performance if needed
  /**
   * Finds an entry index in an `Entries` array.
   *
   * Unlike `Array.prototype.findIndex`, this method returns `null` if the index is not found, not -1.
   */
  findIndex(pred: (entry: Entry<Item>) => boolean): number | null {
    const entry = this.find(pred);

    return entry?.index ?? null;
  }

  // TODO: Improve performance if needed
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
  ): Entries<Item> {
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

    return this;
  }
}
