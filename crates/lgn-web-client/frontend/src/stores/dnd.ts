import { writable } from "svelte/store";
import type { Position } from "../lib/types";

export type Value<Item = unknown> = {
  item: Item;
  mousePosition: Position;
  type: string;
};

/**
 * Keep track of all drag and drop changes and events.
 */
export function createDndStore<Item = unknown>() {
  return writable<Value<Item> | null>(null);
}
