import { Writable } from "../lib/store";
import { Position } from "../lib/types";

export type Value<Item = unknown> = {
  item: Item;
  mousePosition: Position;
  type: string;
};

/**
 * Keep track of all drag and drop changes and events.
 */
export default class<Item = unknown> extends Writable<Value<Item> | null> {
  constructor() {
    super(null);
  }
}
