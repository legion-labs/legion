import type { ScopeDesc } from "@lgn/proto-telemetry/dist/calltree";
import type { Process } from "@lgn/proto-telemetry/dist/process";

import type { ProcessAsyncData } from "../Lib/ProcessAsyncData";
import type { Thread } from "../Lib/Thread";
import type { ThreadBlock } from "../Lib/ThreadBlock";
import type { TimelinePan } from "../Lib/TimelinePan";

export class TimelineState {
  minMs = Infinity;
  maxMs = -Infinity;
  canvasWidth: number;
  threads: Record<string, Thread> = {};
  blocks: Record<string, ThreadBlock> = {};
  eventCount = 0;
  processes: Process[] = [];
  processAsyncData: Record<string, ProcessAsyncData> = {};
  scopes: Record<number, ScopeDesc> = {};
  ready = false;
  beginRange: number | null = null;
  currentSelection: [number, number] | undefined;
  viewRange: [number, number];
  timelinePan: TimelinePan | null = null;
  private timelineStart: number | null;
  private timelineEnd: number | null;
  constructor(canvasWidth: number, start: number | null, end: number | null) {
    this.canvasWidth = canvasWidth;
    this.timelineStart = start;
    this.timelineEnd = end;
    this.viewRange = this.getViewRange();
  }

  getPixelWidthMs(): number {
    const range = this.getViewRange();
    const timeSpan = range[1] - range[0];
    return this.canvasWidth / timeSpan;
  }

  isFullyVisible() {
    if (!this.viewRange) {
      return false;
    }
    return !(
      this.viewRange[0] <= this.minMs && this.viewRange[1] >= this.maxMs
    );
  }

  createdWithParameters() {
    return this.timelineStart && this.timelineEnd;
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

  getMaxRange() {
    return this.maxMs - this.minMs;
  }

  findStreamProcess(streamId: string) {
    const stream = this.threads[streamId].streamInfo;
    return this.processes.find((p) => p.processId === stream.processId);
  }
}
