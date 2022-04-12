import { writable } from "svelte/store";

import type { TimelineState } from "./TimelineState";

export type TimelineStateStore = ReturnType<typeof createTimelineStateStore>;

export function createTimelineStateStore(state: TimelineState) {
  const { subscribe, set, update } = writable(state);

  const keyboardZoom = (positive: boolean) => {
    update((s) => {
      const range = s.getViewRange();
      const length = range[1] - range[0];
      const change = ((positive ? 1 : -1) * length) / 10;
      s.setViewRange([range[0] + change, range[1] - change]);
      return s;
    });
  };

  const keyboardTranslate = (positive: boolean) => {
    update((s) => {
      const range = s.getViewRange();
      const length = range[1] - range[0];
      const delta = ((positive ? 1 : -1) * length) / 10;
      s.setViewRange([range[0] + delta, range[1] + delta]);
      return s;
    });
  };

  const wheelZoom = (event: WheelEvent) => {
    const speed = 0.75;
    const factor = event.deltaY > 0 ? 1.0 / speed : speed;
    update((s) => {
      const range = s.getViewRange();
      const length = range[1] - range[0];
      const newLength = length * factor;
      const pctCursor = event.offsetX / s.canvasWidth;
      const pivot = range[0] + length * pctCursor;
      const result = [
        pivot - newLength * pctCursor,
        pivot + newLength * (1 - pctCursor),
      ];
      s.setViewRange([result[0], result[1]]);
      return s;
    });
  };

  // At some point we could remove the update function to disable arbitrary changes to the store.
  return { subscribe, set, update, keyboardZoom, keyboardTranslate, wheelZoom };
}
