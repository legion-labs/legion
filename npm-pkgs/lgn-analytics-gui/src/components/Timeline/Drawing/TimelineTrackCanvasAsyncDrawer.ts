import type { ProcessAsyncData } from "../Lib/ProcessAsyncData";
import type { TimelineState } from "../Stores/TimelineState";
import { TimelineTrackCanvasBaseDrawer } from "./TimelineTrackCanvasBaseDrawer";
import type { TimelineTrackContext } from "./TimelineTrackContext";

export class TimelineTrackCanvasAsyncDrawer extends TimelineTrackCanvasBaseDrawer {
  private async: ProcessAsyncData;

  constructor(processOffsetMs: number, processAsyncData: ProcessAsyncData) {
    super(processOffsetMs);
    this.async = processAsyncData;
  }

  protected canDraw(): boolean {
    return (this.async.sections?.length ?? 0) > 0;
  }

  protected getPixelRange(ctx: TimelineTrackContext): [number, number] {
    const begin = Math.max(ctx.begin, this.async.minMs + this.processOffsetMs);
    const end = Math.min(ctx.end, this.async.maxMs + this.processOffsetMs);
    const beginTasksPixels = (begin - ctx.begin) * ctx.msToPixelsFactor;
    const endTasksPixels = (end - ctx.begin) * ctx.msToPixelsFactor;
    return [beginTasksPixels, endTasksPixels];
  }

  protected drawImpl(ctx: TimelineTrackContext, state: TimelineState) {
    this.async.sections.forEach((section) => {
      for (
        let trackIndex = 0;
        trackIndex < section.tracks.length;
        trackIndex += 1
      ) {
        const track = section.tracks[trackIndex];
        this.drawSpanTrack(trackIndex, track, ctx, state);
      }
    });
  }
}
