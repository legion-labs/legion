import { Thread } from "./Thread";
import { spanPixelHeight } from "./TimelineValues";

export function getThreadCollapseStyle(maxDepth: number, collapsed: boolean) {
  return collapsed
    ? `max-height:${spanPixelHeight}px`
    : `height:${Math.max(spanPixelHeight, maxDepth * spanPixelHeight)}px`;
}
