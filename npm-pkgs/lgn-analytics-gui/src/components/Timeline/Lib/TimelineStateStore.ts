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
      s.viewRange = [range[0] + change, range[1] - change];
      return s;
    });
  };

  const keyboardTranslate = (positive: boolean) => {
    update((s) => {
      const range = s.getViewRange();
      const length = range[1] - range[0];
      const delta = ((positive ? 1 : -1) * length) / 10;
      s.viewRange = [range[0] + delta, range[1] + delta];
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
      s.viewRange = [result[0], result[1]];
      return s;
    });
  };

  const updateState = (action: (state: TimelineState) => void) => {
    update((s) => {
      action(s);
      return s;
    });
  };

  const setSelection = (range: [number, number]) => {
    updateState((s) => {
      s.currentSelection = range;
    });
  };

  const updateWidth = (width: number) => {
    updateState((s) => {
      s.canvasWidth = width;
    });
  };

  const setViewRange = (range: [number, number]) => {
    updateState((s) => {
      s.viewRange = range;
    });
  };

  const clearSelection = () => {
    updateState((s) => {
      s.beginRange = null;
      s.currentSelection = undefined;
    });
  };

  const startSelection = (x: number) => {
    updateState((s) => {
      s.beginRange = x;
      s.currentSelection = undefined;
    });
  };

  const updateSelection = (x: number) => {
    updateState((s) => {
      if (s.beginRange) {
        const viewRange = s.viewRange;
        const factor = (viewRange[1] - viewRange[0]) / s.canvasWidth;
        const first = viewRange[0] + factor * s.beginRange;
        const second = viewRange[0] + factor * x;
        s.currentSelection = [Math.min(first, second), Math.max(first, second)];
      }
    });
  };

  const applyDrag = (offsetX: number) => {
    updateState((s) => {
      if (!s.timelinePan) {
        s.timelinePan = {
          beginMouseX: offsetX,
          viewRange: [s.viewRange[0], s.viewRange[1]],
        };
      }
      const viewRange = s.timelinePan.viewRange;
      const factor = (viewRange[1] - viewRange[0]) / s.canvasWidth;
      const offsetMs = factor * (s.timelinePan.beginMouseX - offsetX);
      s.viewRange = [viewRange[0] + offsetMs, viewRange[1] + offsetMs];
    });
  };

  const stopDrag = () => {
    updateState((s) => (s.timelinePan = null));
  };

  return {
    subscribe,
    set,
    keyboardZoom,
    keyboardTranslate,
    wheelZoom,
    updateWidth,
    setSelection,
    setViewRange,
    startSelection,
    clearSelection,
    updateSelection,
    applyDrag,
    stopDrag,
    update,
  };
}
