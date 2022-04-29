import type { CumulativeCallGraphComputedBlock } from "@lgn/proto-telemetry/dist/callgraph";

import { CallGraphNode } from "./CallGraphNode";

export class CallGraphFlatData {
  nodes: Map<number, CallGraphNode> = new Map();

  ingestBlock(block: CumulativeCallGraphComputedBlock) {
    for (const node of block.nodes) {
      if (!node.node) continue;
      if (node.node.hash === 0) {
        node.node.hash = block.streamHash;
      }
      for (const caller of node.callers) {
        if (caller.hash === 0) caller.hash = block.streamHash;
      }
      const element = this.nodes.get(node.node.hash);
      if (element) {
        element.ingest(node);
      } else {
        this.nodes.set(node.node.hash, new CallGraphNode(node));
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
