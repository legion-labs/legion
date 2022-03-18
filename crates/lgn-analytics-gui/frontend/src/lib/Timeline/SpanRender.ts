import binarySearch from "binary-search";
import { spanPixelHeight } from "@/lib/Timeline/TimelineValues";
import { SpanTrack } from "@lgn/proto-telemetry/dist/analytics";
import { formatExecutionTime } from "@/lib/format";
import { TimelineCaptionItem } from "@/lib/Timeline/TimelineSpanCaptionItem";
import { ScopeDesc } from "@lgn/proto-telemetry/dist/calltree";

export function drawSpanTrack(
  ctx: CanvasRenderingContext2D,
  scopes: Record<number, ScopeDesc>,
  track: SpanTrack,
  color: string,
  offsetY: number,
  processOffsetMs: number,
  beginViewRange: number,
  endViewRange: number,
  characterWidth: number,
  characterHeight: number,
  msToPixelsFactor: number
) {
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

  for (let spanIndex = firstSpan; spanIndex < lastSpan; spanIndex += 1) {
    const span = track.spans[spanIndex];
    const beginSpan = span.beginMs + processOffsetMs;
    const endSpan = span.endMs + processOffsetMs;

    const beginPixels = (beginSpan - beginViewRange) * msToPixelsFactor;
    const endPixels = (endSpan - beginViewRange) * msToPixelsFactor;
    const callWidth = endPixels - beginPixels;
    if (callWidth < 0.1) {
      continue;
    }
    ctx.fillStyle = color;
    ctx.globalAlpha = span.alpha / 255;
    ctx.fillRect(beginPixels, offsetY, callWidth, spanPixelHeight);
    ctx.globalAlpha = 1.0;

    if (callWidth >= 16) {
      ctx.save();
      ctx.fillStyle = shadeColor(ctx.fillStyle, 1.025);
      ctx.fillRect(beginPixels, offsetY, 2, spanPixelHeight);
      ctx.restore();
    }

    if (span.scopeHash != 0) {
      if (callWidth > characterWidth * 5) {
        // const nbChars = Math.floor(callWidth / characterWidth);

        ctx.fillStyle = "#000000";

        const extraHeight = 0.5 * (spanPixelHeight - characterHeight);
        const { name } = scopes[span.scopeHash];
        // const caption = name + " " + formatExecutionTime(endSpan - beginSpan);

        // ctx.fillText(
        //   caption.slice(0, nbChars),
        //   beginPixels + 5,
        //   offsetY + characterHeight + extraHeight,
        //   callWidth
        // );

        writeText(
          ctx,
          callWidth,
          characterWidth,
          Array.from(getCaptions(name, beginSpan, endSpan)),
          beginPixels + 5,
          offsetY + characterHeight + extraHeight
        );
      }
    }
  }
}

function writeText(
  ctx: CanvasRenderingContext2D,
  width: number,
  characterWidth: number,
  items: TimelineCaptionItem[],
  x: number,
  y: number
) {
  const defaultFillStyle = ctx.fillStyle;
  const defaultFont = ctx.font;
  ctx.save();
  for (const { value, font, color, skippable } of items) {
    ctx.fillStyle = color || defaultFillStyle;
    ctx.font = font || defaultFont;
    const budget = Math.floor(width / characterWidth);
    if (!budget) {
      break;
    }
    if (value.length > budget && skippable) {
      continue;
    }
    const textSlice = value.slice(0, budget);
    ctx.fillText(textSlice, x, y);
    const size = ctx.measureText(textSlice).width;
    x += size;
    width -= size;
  }
  ctx.restore();
}

function* getCaptions(
  caption: string,
  beginSpan: number,
  endSpan: number
): Generator<TimelineCaptionItem> {
  const mainColor = "#000000";
  const subColor = "#4d4d4d";
  const defaultFont = "15px arial";
  const split = caption.split("::");
  if (split.length > 1) {
    const first = split.shift();
    yield { value: first ?? "", font: defaultFont, color: subColor };
    let current = null;
    while ((current = split.shift())) {
      yield {
        value: `::${current}`,
        font: defaultFont,
        color: split.length > 0 ? subColor : mainColor,
      };
    }
  } else {
    yield { value: caption, color: mainColor };
  }
  yield {
    value: `  (${formatExecutionTime(endSpan - beginSpan)})`,
    color: subColor,
    font: "12px arial",
    skippable: true,
  };
}

function shadeColor(color: string, decimal: number): string {
  const base = color.startsWith("#") ? 1 : 0;

  let r = parseInt(color.substring(base, 3), 16);
  let g = parseInt(color.substring(base + 2, 5), 16);
  let b = parseInt(color.substring(base + 4, 7), 16);

  r = Math.round(r / decimal);
  g = Math.round(g / decimal);
  b = Math.round(b / decimal);

  r = r < 255 ? r : 255;
  g = g < 255 ? g : 255;
  b = b < 255 ? b : 255;

  const rr =
    r.toString(16).length === 1 ? `0${r.toString(16)}` : r.toString(16);
  const gg =
    g.toString(16).length === 1 ? `0${g.toString(16)}` : g.toString(16);
  const bb =
    b.toString(16).length === 1 ? `0${b.toString(16)}` : b.toString(16);

  return `#${rr}${gg}${bb}`;
}
