import { Thread } from "./Thread";
import { ThreadBlock } from "./ThreadBlock";

export class TimelineState {
  minMs = Infinity;
  maxMs = -Infinity;
  threads: Record<string, Thread> = {};
  blocks: Record<string, ThreadBlock> = {};
  eventCount = 0;
  timelineStart: number | undefined;
  timelineEnd: number | undefined;
  private viewRange: [number, number] | null = null;
  constructor(start: number | undefined, end: number | undefined) {
    this.timelineStart = start;
    this.timelineEnd = end;
  }

  setViewRange(range: [number, number]) {
    this.viewRange = range;
  }

  getViewRange(): [number, number] {
    if (this.viewRange) {
      return this.viewRange;
    }
    let start = this.minMs;
    if (this.timelineStart) {
      start = this.timelineStart;
    }
    let end = this.maxMs;
    if (this.timelineEnd) {
      end = this.timelineEnd;
    }
    return [start, end];
  }
}
