declare module "@sveltejs/svelte-virtual-list" {
  import { SvelteComponentTyped } from "svelte";

  export interface VirtualListProps<T> {
    items: T[];
    start?: number;
    end?: number;
    itemHeight?: number;
  }

  interface DefaultSlot<T> {
    item: T;
  }

  export interface VirtualListSlots<T> {
    default: DefaultSlot<T>;
  }

  export default class VirtualList<T> extends SvelteComponentTyped<
    VirtualListProps<T>,
    Record<string, never>,
    VirtualListSlots<T>
  > {}
}
