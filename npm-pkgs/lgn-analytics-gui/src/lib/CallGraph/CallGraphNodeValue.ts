import type { CumulativeStats } from "@lgn/proto-telemetry/dist/callgraph";

export class CallGraphNodeValue {
  acc = 0;
  avg = 0;
  count = 0;
  sd = 0;
  sqr = 0;
  childSum = 0;
  min = Infinity;
  max = -Infinity;

  constructor(stats: CumulativeStats | null) {
    if (stats) {
      this.accumulateStats(stats);
    }
  }

  accumulateStats(input: CumulativeStats) {
    this.min = Math.min(this.min, input.min);
    this.max = Math.max(this.max, input.max);
    this.count += input.count;
    this.acc += input.sum;
    this.sqr += input.sumSqr;
    this.avg = this.acc / this.count;
    this.sd = Math.sqrt(this.sqr / this.count - Math.pow(this.avg, 2));
    this.childSum += input.childSum;
  }
}
