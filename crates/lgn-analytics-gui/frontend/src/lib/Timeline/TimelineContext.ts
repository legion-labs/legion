import { writable } from "svelte/store";

export class TimelineContextState {
  search = writable<string>();
}

export const TimelineContext = new TimelineContextState();
