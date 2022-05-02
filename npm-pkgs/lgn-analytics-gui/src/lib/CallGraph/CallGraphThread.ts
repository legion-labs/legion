import type { CumulativeCallGraphComputedBlock } from "@lgn/proto-telemetry/dist/callgraph";

import { CallGraphNode } from "./CallGraphNode";

export class CallGraphThread {
  streamHash: number;
  data: Map<number, CallGraphNode> = new Map();
  constructor(block: CumulativeCallGraphComputedBlock) {
    this.streamHash = block.streamHash;
    this.ingestBlock(block);
  }

  ingestBlock(block: CumulativeCallGraphComputedBlock) {
    for (const node of block.nodes) {
      if (!node.node) {
        continue;
      }
      if (node.node.hash === 0) node.node.hash = block.streamHash;
      const item = this.data.get(node.node.hash);
      if (item) {
        item.ingest(node);
      } else {
        this.data.set(node.node.hash, new CallGraphNode(node, false));
      }
    }
  }
}
