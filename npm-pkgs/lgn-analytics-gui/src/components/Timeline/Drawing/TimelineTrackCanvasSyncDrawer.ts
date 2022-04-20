import { findBestLod } from "@/lib/time";

import type { Thread } from "../Lib/Thread";
import type { TimelineState } from "../Stores/TimelineState";
import { TimelineTrackCanvasBaseDrawer } from "./TimelineTrackCanvasBaseDrawer";
import type { TimelineTrackContext } from "./TimelineTrackContext";

export class TimelineTrackCanvasSyncDrawer extends TimelineTrackCanvasBaseDrawer {
  private thread: Thread;

  constructor(processOffsetMs: number, thread: Thread) {
    super(processOffsetMs);
    this.thread = thread;
  }

  protected canDraw(): boolean {
    return (this.thread.block_ids?.length ?? 0) > 0;
  }

  protected getPixelRange(ctx: TimelineTrackContext): [number, number] {
    const begin = Math.max(ctx.begin, this.thread.minMs + this.processOffsetMs);
    const end = Math.min(ctx.end, this.thread.maxMs + this.processOffsetMs);
    const beginThreadPixels = (begin - ctx.begin) * ctx.msToPixelsFactor;
    const endThreadPixels = (end - ctx.begin) * ctx.msToPixelsFactor;
    return [beginThreadPixels, endThreadPixels];
  }

  protected drawImpl(ctx: TimelineTrackContext, state: TimelineState) {
    const blocks = state.blocks;
    this.thread.block_ids.forEach((block_id) => {
      const block = blocks[block_id];
      const lodToRender = !this.canvas
        ? null
        : findBestLod(this.canvas.width, [ctx.begin, ctx.end], block);

      if (block.beginMs > ctx.end || block.endMs < ctx.begin || !lodToRender) {
        return;
      }

      for (
        let trackIndex = 0;
        trackIndex < lodToRender.tracks.length;
        trackIndex += 1
      ) {
        const track = lodToRender.tracks[trackIndex];
        this.drawSpanTrack(trackIndex, track, ctx, state);
      }
    });
  }
}
