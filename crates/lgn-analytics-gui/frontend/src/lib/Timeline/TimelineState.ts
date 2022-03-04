import { Thread } from "./Thread";

export class TimelineState {
  minMs = Infinity;
  maxMs = -Infinity;
  threads: Record<string, Thread> = {};
}
