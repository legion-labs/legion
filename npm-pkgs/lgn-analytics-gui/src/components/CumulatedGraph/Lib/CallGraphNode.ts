import type {
  CumulativeCallGraphEdge,
  CumulativeComputedCallGraphNode,
} from "@lgn/proto-telemetry/dist/callgraph";
import { CallGraphNodeValue } from "./CallGraphNodeValue";

export class CallGraphNode {
  children: Map<number, CallGraphNodeValue> = new Map();
  parent: Map<number, CallGraphNodeValue> = new Map();
  value: CallGraphNodeValue = new CallGraphNodeValue(null);
  hash: number;
  constructor(node: CumulativeComputedCallGraphNode) {
    this.hash = node.node?.hash ?? 0;
    this.ingest(node);
  }

  ingest(input: CumulativeComputedCallGraphNode) {
    if (input.node) {
      this.value.accumulateEdge(input.node);
    }
    this.collectionIngest(this.children, input.callees);
    this.collectionIngest(this.parent, input.callers);
  }

  private collectionIngest(
    map: Map<number, CallGraphNodeValue>,
    edges: CumulativeCallGraphEdge[]
  ) {
    for (const edge of edges) {
      const item = map.get(edge.hash);
      if (item) {
        item.accumulateEdge(edge);
      } else {
        map.set(edge.hash, new CallGraphNodeValue(edge));
      }
    }
  }
}
