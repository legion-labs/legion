import { Writable } from "svelte/store";

export type TimelineContext = {
  search: Writable<string>;
};
