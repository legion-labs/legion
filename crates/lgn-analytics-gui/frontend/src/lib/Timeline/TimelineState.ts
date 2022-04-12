import type { ScopeDesc } from "@lgn/proto-telemetry/dist/calltree";
import type { Process } from "@lgn/proto-telemetry/dist/process";

import type { SelectionState } from "../time_range_selection";
import { NewSelectionState } from "../time_range_selection";
import type { ProcessAsyncData } from "./ProcessAsyncData";
import type { Thread } from "./Thread";
import type { ThreadBlock } from "./ThreadBlock";

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
  selectionState: SelectionState;
  currentSelection: [number, number] | undefined;
  private timelineStart: number | null;
  private timelineEnd: number | null;
  private viewRange: [number, number] | null = null;
  constructor(canvasWidth: number, start: number | null, end: number | null) {
    this.canvasWidth = canvasWidth;
    this.timelineStart = start;
    this.timelineEnd = end;
    this.selectionState = NewSelectionState();
  }

  getPixelWidthMs(): number {
    const range = this.getViewRange();
    const timeSpan = range[1] - range[0];
    return this.canvasWidth / timeSpan;
  }

  setViewRange(range: [number, number]) {
    this.viewRange = range;
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
