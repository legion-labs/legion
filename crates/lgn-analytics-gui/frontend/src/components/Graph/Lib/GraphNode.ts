import type { CumulativeCallGraphNode } from "@lgn/proto-telemetry/dist/analytics";

export class GraphNode {
  private hash: number;
  constructor(node: CumulativeCallGraphNode) {
    this.hash = node.hash;
  }
}
