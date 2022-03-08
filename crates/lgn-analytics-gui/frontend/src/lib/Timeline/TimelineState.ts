import { ScopeDesc } from "@lgn/proto-telemetry/dist/calltree";
import { Process } from "@lgn/proto-telemetry/dist/process";
import { NewSelectionState, SelectionState } from "../time_range_selection";
import { zoomHorizontalViewRange } from "../zoom";
import { Thread } from "./Thread";
import { ThreadBlock } from "./ThreadBlock";

export class TimelineState {
  minMs = Infinity;
  maxMs = -Infinity;
  threads: Record<string, Thread> = {};
  blocks: Record<string, ThreadBlock> = {};
  eventCount = 0;
  processes: Process[] = [];
  scopes: Record<number, ScopeDesc> = {};
  ready = false;
  selectionState: SelectionState;
  currentSelection: [number, number] | undefined;
  private timelineStart: number | undefined;
  private timelineEnd: number | undefined;
  private viewRange: [number, number] | null = null;
  constructor(start: number | undefined, end: number | undefined) {
    this.timelineStart = start;
    this.timelineEnd = end;
    this.selectionState = NewSelectionState();
  }

  setViewRange(range: [number, number]) {
    this.viewRange = range;
  }

  setViewRangeFromWheel(
    viewRange: [number, number],
    canvasWidth: number,
    wheelEvent: WheelEvent
  ) {
    this.setViewRange(
      zoomHorizontalViewRange(viewRange, canvasWidth, wheelEvent)
    );
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

  findStreamProcess(streamId: string) {
    const stream = this.threads[streamId].streamInfo;
    return this.processes.find((p) => p.processId === stream.processId);
  }
}
