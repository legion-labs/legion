import { Writable } from "@lgn/web-client/src/lib/store";
import { TimelineState } from "./TimelineState";

export class TimelineStateStore extends Writable<TimelineState> {
  constructor(state: TimelineState) {
    super(state);
  }
}
