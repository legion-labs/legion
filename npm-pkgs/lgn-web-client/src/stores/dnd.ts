import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

import type { NonEmptyArray } from "../lib/array";
import type { Position } from "../lib/types";

export type DndValue<Item = unknown> = {
  item: Item;
  mousePosition: Position;
  type: string;
};

export type DndStore<Item = unknown> = Writable<NonEmptyArray<
  DndValue<Item>
> | null>;

/**
 * Keep track of all drag and drop changes and events.
 */
export function createDndStore<Item = unknown>(): DndStore<Item> {
  return writable(null);
}
