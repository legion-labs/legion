import type {
  CumulativeCallGraphEdge,
  CumulativeComputedCallGraphNode,
} from "@lgn/proto-telemetry/dist/callgraph";

import { CallGraphNodeValue } from "./CallGraphNodeValue";

export class CallGraphNode {
  children: Map<number, CallGraphNodeValue> = new Map();
  parents: Map<number, CallGraphNodeValue> = new Map();
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
    this.children = this.collectionIngest(this.children, input.callees, (e) =>
      e.sort((a, b) => b[1].acc - a[1].acc)
    );
    this.parents = this.collectionIngest(this.parents, input.callers, (e) =>
      e.sort((a, b) => b[1].childSum - a[1].childSum)
    );
  }

  private collectionIngest(
    map: Map<number, CallGraphNodeValue>,
    edges: CumulativeCallGraphEdge[],
    sort: (
      map: [number, CallGraphNodeValue][]
    ) => [number, CallGraphNodeValue][]
  ) {
    for (const edge of edges) {
      const item = map.get(edge.hash);
      if (item) {
        item.accumulateEdge(edge);
      } else {
        map.set(edge.hash, new CallGraphNodeValue(edge));
      }
    }
    return new Map(sort([...map]));
  }
}
