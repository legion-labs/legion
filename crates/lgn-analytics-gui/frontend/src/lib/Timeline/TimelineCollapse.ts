import { Thread } from "./Thread";
import { spanPixelHeight } from "./TimelineValues";

export function getThreadCollapseStyle(thread: Thread, collapsed: boolean) {
  return collapsed
    ? `max-height:${spanPixelHeight}px`
    : `height:${Math.max(
        spanPixelHeight,
        thread.maxDepth * spanPixelHeight
      )}px`;
}
