import { components } from "./path";

export type Entry<Item> = {
  item: Item;
  entries?: Entries<Item> | null;
};

export type Entries<Item> = Record<string, Entry<Item>>;

// TODO: Improve performance if needed
/**
 * Takes a whole `Entries` object and a function called for each entry in this object.
 *
 * If the function returns `null` nothing happens,
 * if an item and/or a name is returned, then the entry will be updated.
 */
export function updateEntry<Item>(
  entries: Entries<Item>,
  updateItem: (
    name: string,
    item: Item
  ) => { name: string } | { item: Item } | { name: string; item: Item } | null
): Entries<Item> {
  return Object.fromEntries(
    Object.entries(entries).map(([name, entry]) => {
      const updatedItem = updateItem(name, entry.item);

      if (updatedItem) {
        return [
          ("name" in updatedItem && updatedItem.name.trim()) || name,
          "item" in updatedItem ? { ...entry, item: updatedItem.item } : entry,
        ];
      }

      if (!entry.entries) {
        return [name, entry];
      }

      return [
        name,
        {
          ...entry,
          entries: updateEntry(entry.entries, updateItem),
        },
      ];
    })
  );
}

// TODO: Improve performance if needed
/**
 * Build an `Entries` object from any flat arrays of object.
 * Objects must contain a `path` attribute.
 *
 * ## Example
 *
 * ```typescript
 * const entries = unflatten([
 *   { path: "/foo/bar", value: "hello" },
 *   { path: "/foo/baz", value: "another hello" },
 *   { path: "/foo", value: "another value" },
 *   { path: "baz", value: "a baz value" },
 *   { path: "/foo/bar/baz", value: "another baz value" },
 * ]);
 *
 * const expectedEntries = {
 *   foo: {
 *     item: null,
 *     entries: {
 *       bar: {
 *         item: { path: "/foo/bar", value: "hello" },
 *         entries: {
 *           baz: {
 *             item: { path: "/foo/bar/baz", value: "another baz value" },
 *           },
 *         },
 *       },
 *       baz: {
 *         item: { path: "/foo/baz", value: "another hello" },
 *       },
 *     },
 *   },
 *   baz: {
 *     item: { path: "baz", value: "a baz value" },
 *   },
 * };
 *
 * // Given a `deepEqual` function:
 * console.assert(deepEqual(entries, expectedEntries));
 * ```
 */
export function unflatten<Item extends { path: string }>(
  items: Item[]
): Entries<Item | null> {
  if (!items.length) {
    return {};
  }

  function buildEntriesFromPathComponents(
    [pathComponent, ...otherPathComponents]: string[],
    item: Item,
    entries: Entries<Item | null>
  ): Entries<Item | null> {
    const entry =
      pathComponent in entries ? entries[pathComponent] : { item: null };

    return {
      ...entries,
      [pathComponent]: otherPathComponents.length
        ? {
            ...entry,
            entries: buildEntriesFromPathComponents(
              otherPathComponents,
              item,
              entry.entries || {}
            ),
          }
        : { ...entry, item },
    };
  }

  function buildEntriesFromItems(
    [item, ...otherItems]: Item[],
    entries: Entries<Item | null> = {}
  ): Entries<Item | null> {
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

  return buildEntriesFromItems(items);
}
