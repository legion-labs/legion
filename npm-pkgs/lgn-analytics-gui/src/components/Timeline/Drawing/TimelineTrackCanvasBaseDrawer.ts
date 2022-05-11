import binarySearch from "binary-search";

import type { SpanTrack } from "@lgn/proto-telemetry/dist/span";

import { spanPixelHeight } from "@/components/Timeline/Values/TimelineValues";
import { formatExecutionTime } from "@/lib/format";

import type { TimelineState } from "../Stores/TimelineState";
import type { TimelineTrackContext } from "./TimelineTrackContext";

enum CaptionType {
  Main,
  Sub,
  Annotation,
}

export abstract class TimelineTrackCanvasBaseDrawer {
  protected canvas: HTMLCanvasElement | undefined;
  protected ctx: CanvasRenderingContext2D | undefined;
  protected processOffsetMs: number;
  constructor(processOffsetMs: number) {
    this.processOffsetMs = processOffsetMs;
  }

  protected abstract canDraw(): boolean;

  protected abstract drawImpl(
    ctx: TimelineTrackContext,
    state: TimelineState
  ): void;

  protected abstract getPixelRange(ctx: TimelineTrackContext): [number, number];

  initialize(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
    this.canvas = canvas;
    this.ctx = ctx;
  }

  draw(search: string, state: TimelineState) {
    if (!this.canvas || !this.ctx) {
      return;
    }

    const canvasWidth = this.canvas.clientWidth;
    const canvasHeight = this.canvas.clientHeight;
    this.ctx.fillStyle = "#2e2e2e";
    this.ctx.fillRect(0, 0, canvasWidth, canvasHeight);

    if (!this.canDraw()) {
      return;
    }

    const [begin, end] = state.viewRange;
    const invTimeSpan = 1.0 / (end - begin);
    const msToPixelsFactor = invTimeSpan * canvasWidth;
    const context = { begin, end, msToPixelsFactor, search };
    const pixelRange = this.getPixelRange(context);

    this.ctx.fillStyle = "#1a1a1a";
    this.ctx.fillRect(
      pixelRange[0],
      0,
      pixelRange[1] - pixelRange[0],
      canvasHeight
    );

    this.drawImpl(context, state);
  }

  protected drawSpanTrack(
    trackIndex: number,
    track: SpanTrack,
    timelineTrackContext: TimelineTrackContext,
    state: TimelineState
  ) {
    if (!this.ctx) {
      return;
    }
    const processOffsetMs = this.processOffsetMs;
    const beginViewRange = timelineTrackContext.begin;
    const endViewRange = timelineTrackContext.end;
    const msToPixelsFactor = timelineTrackContext.msToPixelsFactor;
    const search = timelineTrackContext.search;

    let firstSpan = binarySearch(
      track.spans,
      beginViewRange - processOffsetMs,
      function (span, needle) {
        if (span.endMs < needle) {
          return -1;
        }
        if (span.beginMs > needle) {
          return 1;
        }
        return 0;
      }
    );
    if (firstSpan < 0) {
      firstSpan = ~firstSpan;
    }

    let lastSpan = binarySearch(
      track.spans,
      endViewRange - processOffsetMs,
      function (span, needle) {
        if (span.beginMs < needle) {
          return -1;
        }
        if (span.endMs > needle) {
          return 1;
        }
        return 0;
      }
    );
    if (lastSpan < 0) {
      lastSpan = ~lastSpan;
    }

    const ctx = this.ctx;
    ctx.font = this.captionFont(CaptionType.Main);
    const testString = "<>_w";
    const testTextMetrics = ctx.measureText(testString);
    const characterWidth = testTextMetrics.width / testString.length;
    const characterHeight = testTextMetrics.actualBoundingBoxAscent;
    const offsetY = trackIndex * spanPixelHeight;
    const color = this.spanColor(trackIndex);
    ctx.save();
    for (let spanIndex = firstSpan; spanIndex < lastSpan; ++spanIndex) {
      const span = track.spans[spanIndex];
      const beginSpan = span.beginMs + processOffsetMs;
      const endSpan = span.endMs + processOffsetMs;

      const beginPixels = Math.max(
        0,
        (beginSpan - beginViewRange) * msToPixelsFactor
      );
      const endPixels = (endSpan - beginViewRange) * msToPixelsFactor;
      const callWidth = endPixels - beginPixels;
      // if less than half a pixel, clip it
      if (callWidth < 0.5) {
        continue;
      }
      ctx.globalAlpha = span.alpha / 255;
      if (span.scopeHash !== 0) {
        let name = "<unknown_scope>";
        const scope = state.scopes[span.scopeHash];
        if (scope !== undefined) {
          name = scope.name;
        }
        ctx.fillStyle =
          search && name.toLowerCase().includes(search.toLowerCase())
            ? "#FFEE59"
            : this.spanColor(span.scopeHash);

        // To visually separate consecutive spans, offset them by a pixel
        // unless smaller than 2 pixels
        if (callWidth > 2) {
          ctx.fillRect(
            beginPixels + 1,
            offsetY,
            callWidth - 1,
            spanPixelHeight
          );
        } else {
          ctx.fillRect(beginPixels, offsetY, callWidth, spanPixelHeight);
        }

        // this test is done again within the writeCaption function
        // need to profile if it is faster to do it here or not
        // TODO: #1713 Remove double check of caption
        if (callWidth > characterWidth * 2) {
          const extraHeight = 0.5 * (spanPixelHeight - characterHeight);
          this.writeSpanCaption(
            name,
            ctx,
            callWidth,
            characterWidth,
            beginSpan,
            endSpan,
            beginPixels,
            offsetY + characterHeight + extraHeight
          );
        }
      } else {
        ctx.fillStyle = color;
        ctx.fillRect(beginPixels, offsetY, callWidth, spanPixelHeight);
      }
      ctx.globalAlpha = 1.0;
    }
    ctx.restore();
  }

  private spanColor(hash: number): string {
    const colors = [
      "#57CF86",
      "#FFC464",
      "#F97577",
      "#CA8AF8",
      "#63AAFF",
      "#D9DAE4",
    ];
    return colors[hash % colors.length];
  }

  // For correctness of the code below, the font must be monospaced
  private captionFont(captionType: CaptionType): string {
    switch (captionType) {
      case CaptionType.Main: {
        return "500 13px Roboto Mono";
      }
      case CaptionType.Sub: {
        return "13px Roboto Mono";
      }
      case CaptionType.Annotation: {
        return "italic 11px Roboto Mono";
      }
    }
  }

  private writeSpanCaption(
    caption: string,
    ctx: CanvasRenderingContext2D,
    width: number,
    characterWidth: number,
    beginSpan: number,
    endSpan: number,
    x: number,
    y: number
  ) {
    const captionBudget = width / characterWidth - 2;
    if (captionBudget < 0) {
      return;
    }

    // we truncate the caption if it's too long
    // if less than 2 character, we just clip it
    // if more than 2 characters, we switch the 2 first chars by ..
    if (caption.length > captionBudget) {
      if (captionBudget <= 2) {
        caption = caption.slice(caption.length - captionBudget);
      } else {
        caption = ".." + caption.slice(caption.length - captionBudget + 2);
      }
    }

    ctx.fillStyle = "#000000";
    const lastSeparator = caption.lastIndexOf("::");
    // we start at the half width of a character mark
    x += characterWidth / 2;
    // to keep it simple if there is a scope in the remaining caption, we
    // display the fist part with the sub font and the last part with the main font
    if (lastSeparator !== -1) {
      const mainCaption = caption.slice(lastSeparator + 2, caption.length);
      const subCaption = caption.slice(0, lastSeparator + 2);
      ctx.font = this.captionFont(CaptionType.Sub);
      ctx.fillText(subCaption, x, y);
      ctx.font = this.captionFont(CaptionType.Main);
      ctx.fillText(mainCaption, x + subCaption.length * characterWidth, y);
    } else {
      ctx.font = this.captionFont(CaptionType.Main);
      ctx.fillText(caption, x, y);
    }

    const remainingBudget = captionBudget - caption.length;
    // quick discard to display the annotation
    if (remainingBudget > 4) {
      const timing = `  ${formatExecutionTime(endSpan - beginSpan)}`;
      if (timing.length <= remainingBudget) {
        ctx.font = this.captionFont(CaptionType.Annotation);
        ctx.fillText(timing, x + caption.length * characterWidth, y);
      }
    }
  }
}
