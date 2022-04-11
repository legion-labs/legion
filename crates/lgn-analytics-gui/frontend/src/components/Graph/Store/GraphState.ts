import { derived, get, writable } from "svelte/store";

import type { NodeStateStore } from "./GraphStateStore";

export class GraphState {
  Nodes: Map<number, NodeStateStore> = new Map();
  Store = writable<Map<number, NodeStateStore>>(this.Nodes);
  Roots: number[] = [];
  Max = derived(this.Store, (s) =>
    Math.max(...Array.from(s).map((m) => get(m[1]).acc))
  );
  reset() {
    this.Nodes = new Map();
    this.tick();
  }
  tick() {
    const sorted = Array.from(this.Nodes).sort(
      (lhs, rhs) => get(rhs[1]).acc - get(lhs[1]).acc
    );
    this.Nodes = new Map(sorted);
    this.Store.set(this.Nodes);
  }
}
