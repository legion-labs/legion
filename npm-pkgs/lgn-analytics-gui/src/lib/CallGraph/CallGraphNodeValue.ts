import type { CumulativeCallGraphEdge } from "@lgn/proto-telemetry/dist/callgraph";

export class CallGraphNodeValue {
  acc = 0;
  avg = 0;
  count = 0;
  sd = 0;
  sqr = 0;
  childSum = 0;
  min = Infinity;
  max = -Infinity;
  private isRootNode: boolean;

  constructor(edge: CumulativeCallGraphEdge | null, isRootNode: boolean) {
    this.isRootNode = isRootNode;
    if (edge) {
      this.accumulateEdge(edge);
    }
  }

  accumulateEdge(input: CumulativeCallGraphEdge) {
    this.min = Math.min(this.min, input.min);
    this.max = Math.max(this.max, input.max);
    if (this.isRootNode) {
      this.count = 1;
    } else {
      this.count += input.count;
    }
    this.acc += input.sum;
    this.sqr += input.sumSqr;
    this.avg = this.acc / this.count;
    this.sd = Math.sqrt(this.sqr / this.count - Math.pow(this.avg, 2));
    this.childSum += input.childSum;
  }
}
