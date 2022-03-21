import { writable } from "svelte/store";
import type { TimelineState } from "./TimelineState";

export type TimelineStateStore = ReturnType<typeof createTimelineStateStore>;

export function createTimelineStateStore(state: TimelineState) {
  return writable(state);
}
