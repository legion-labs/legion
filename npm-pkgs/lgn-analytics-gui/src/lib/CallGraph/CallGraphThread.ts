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
      if (!node.stats) {
        continue;
      }
      if (node.stats.hash === 0) {
        node.stats.hash = block.streamHash;
      }
      const item = this.data.get(node.stats.hash);
      if (item) {
        item.ingest(node);
      } else {
        this.data.set(node.stats.hash, new CallGraphNode(node));
      }
    }
  }
}
