import { Thread } from "./Thread";
import { ThreadBlock } from "./ThreadBlock";

export class TimelineState {
  minMs = Infinity;
  maxMs = -Infinity;
  threads: Record<string, Thread> = {};
  blocks: Record<string, ThreadBlock> = {};
}
