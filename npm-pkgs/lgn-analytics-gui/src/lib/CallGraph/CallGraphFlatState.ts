import type { CumulativeCallGraphComputedBlock } from "@lgn/proto-telemetry/dist/callgraph";

import { CallGraphNode } from "./CallGraphNode";
import { CallGraphState } from "./CallGraphState";

export class CallGraphFlatState extends CallGraphState {
  nodes: Map<number, CallGraphNode> = new Map();
  roots: number[] = [];

  clear(): void {
    this.nodes = new Map();
  }

  ingestBlockImpl(block: CumulativeCallGraphComputedBlock) {
    for (const node of block.nodes) {
      if (!node.node) continue;
      const isRoot = node.node.hash === 0;
      if (isRoot) {
        node.node.hash = block.streamHash;
      }
      for (const caller of node.callers) {
        if (caller.hash === 0) caller.hash = block.streamHash;
      }
      const element = this.nodes.get(node.node.hash);
      if (element) {
        element.ingest(node);
      } else {
        this.nodes.set(node.node.hash, new CallGraphNode(node, isRoot));
      }
    }
  }

  getMax() {
    let number = -Infinity;
    for (const node of this.nodes.values()) {
      number = Math.max(node.value.acc, number);
    }
    return number;
  }
}
