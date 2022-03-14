import { writable } from "svelte/store";
import { TimelineState } from "./TimelineState";

export type TimelineStateStore = ReturnType<typeof createTimelineStateStore>;

export function createTimelineStateStore(state: TimelineState) {
  return writable(state);
}
