import { writable } from "svelte/store";

import type {
  CallTreeNode,
  ScopeDesc,
} from "@lgn/proto-telemetry/dist/calltree";

import type { GraphState } from "./GraphState";
import { GraphStateNode } from "./GraphStateNode";

export const scopeStore = writable<Record<number, ScopeDesc>>({});

export type NodeStateStore = ReturnType<typeof getGraphStateStore>;

export function getGraphStateStore(
  hash: number,
  beginMs: number,
  endMs: number,
  graphState: GraphState
) {
  const state = new GraphStateNode(hash, beginMs, endMs, graphState);

  const { subscribe, update } = writable<GraphStateNode>(state);

  const updateState = (action: (state: GraphStateNode) => void) => {
    update((s) => {
      action(s);
      return s;
    });
  };

  const registerSelfCall = (node: CallTreeNode, parent: CallTreeNode | null) =>
    updateState((s) => s.registerSelf(node, parent));

  const registerChildCall = (node: CallTreeNode) =>
    updateState((s) => s.registerChild(node));

  const updateRange = (range: { begin: number; end: number }) => {
    updateState((s) => {
      s.min = range.begin;
      s.max = range.end;
    });
  };

  return { subscribe, registerChildCall, registerSelfCall, updateRange };
}
