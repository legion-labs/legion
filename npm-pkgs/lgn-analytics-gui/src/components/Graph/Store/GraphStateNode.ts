import type { CallTreeNode } from "@lgn/proto-telemetry/dist/calltree";

import type { GraphState } from "./GraphState";

export class GraphStateNode {
  hash = 0;
  acc = 0;
  avg = 0;
  count = 0;
  sd = 0;
  sqr = 0;
  min = Infinity;
  max = -Infinity;
  childWeight = 0;
  children: Map<number, GraphStateNode> = new Map();
  parents: Map<number, GraphStateNode> = new Map();
  private beginMs;
  private endMs;
  private isRoot;
  private graphState: GraphState;
  constructor(
    hash: number,
    beginMs: number,
    endMs: number,
    graphState: GraphState,
    isRoot?: boolean
  ) {
    this.graphState = graphState;
    this.isRoot = graphState.Roots.includes(hash) && isRoot;
    this.hash = hash;
    this.beginMs = beginMs;
    this.endMs = endMs;
  }

  registerSelf(self: CallTreeNode, parent: CallTreeNode | null) {
    this.ingestSelf(self);
    if (parent) {
      const weight = self.endMs - self.beginMs;
      this.registerParent(parent, weight);
    }
  }

  registerParent(parent: CallTreeNode, weight: number) {
    if (!this.parents.has(parent.hash)) {
      this.parents.set(
        parent.hash,
        new GraphStateNode(
          parent.hash,
          this.beginMs,
          this.endMs,
          this.graphState,
          false
        )
      );
    }
    const parentState = this.parents.get(parent.hash);
    if (parentState) {
      parentState.ingestSelf(parent);
      parentState.childWeight += weight;
    }
  }

  registerChild(child: CallTreeNode) {
    if (child.endMs < this.beginMs || child.beginMs > this.endMs) {
      return;
    }

    if (!this.children.has(child.hash)) {
      this.children.set(
        child.hash,
        new GraphStateNode(
          child.hash,
          this.beginMs,
          this.endMs,
          this.graphState,
          false
        )
      );
    }
    const childState = this.children.get(child.hash);
    if (childState) {
      childState.ingestSelf(child);
    }
  }

  private ingestSelf(node: CallTreeNode) {
    const begin = Math.max(node.beginMs, this.beginMs);
    const end = Math.min(node.endMs, this.endMs);
    const timeMs = end - begin;
    this.count = this.isRoot ? 1 : this.count + 1;
    this.min = Math.min(this.min, timeMs);
    this.max = Math.max(this.max, timeMs);
    this.acc += timeMs;
    this.sqr += Math.pow(timeMs, 2);
    this.avg = this.acc / this.count;
    this.sd = Math.sqrt(this.sqr / this.count - Math.pow(this.avg, 2));
  }
}
