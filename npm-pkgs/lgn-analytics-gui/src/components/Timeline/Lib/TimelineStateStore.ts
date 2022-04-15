import { writable } from "svelte/store";

import type {
  AsyncSpansReply,
  BlockAsyncEventsStatReply,
  BlockSpansReply,
} from "@lgn/proto-telemetry/dist/analytics";
import type { BlockMetadata } from "@lgn/proto-telemetry/dist/block";
import type { ScopeDesc } from "@lgn/proto-telemetry/dist/calltree";
import type { Process } from "@lgn/proto-telemetry/dist/process";
import type { Stream } from "@lgn/proto-telemetry/dist/stream";

import { LODState } from "./LodState";
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

  const addProcessAsyncBlock = (processId: string) => {
    updateState((s) => {
      s.processAsyncData[processId] = {
        processId: processId,
        maxDepth: 0,
        minMs: Infinity,
        maxMs: -Infinity,
        blockStats: {},
        sections: [],
      };
    });
  };

  const addBlock = (
    beginMs: number,
    endMs: number,
    block: BlockMetadata,
    streamId: string
  ) => {
    updateState((s) => {
      s.minMs = Math.min(s.minMs, beginMs);
      s.maxMs = Math.max(s.maxMs, endMs);
      s.eventCount += block.nbObjects;
      s.threads[streamId].block_ids.push(block.blockId);
      s.blocks[block.blockId] = {
        blockDefinition: block,
        beginMs: beginMs,
        endMs: endMs,
        lods: [],
      };
    });
  };

  const addBlockData = (response: BlockSpansReply) => {
    updateState((s) => {
      if (!s.ready) {
        s.ready = true;
      }
      addScopes(response.scopes);
      const block = s.blocks[response.blockId];
      const thread = s.threads[block.blockDefinition.streamId];
      const blockLod = response.lod;
      if (blockLod) {
        thread.maxDepth = Math.max(thread.maxDepth, blockLod.tracks.length);
        thread.minMs = Math.min(thread.minMs, response.beginMs);
        thread.maxMs = Math.max(thread.maxMs, response.endMs);
        thread.block_ids.push(response.blockId);
        block.lods[blockLod.lodId].tracks = blockLod.tracks;
      }
      return s;
    });
  };

  const addAsyncBlockData = (
    processId: string,
    data: BlockAsyncEventsStatReply
  ) => {
    updateState((s) => {
      const asyncData = s.processAsyncData[processId];
      asyncData.minMs = Math.min(asyncData.minMs, data.beginMs);
      asyncData.maxMs = Math.max(asyncData.maxMs, data.endMs);
      asyncData.blockStats[data.blockId] = data;
    });
  };

  const addAsyncData = (
    processId: string,
    reply: AsyncSpansReply,
    sectionNumber: number
  ) => {
    updateState((s) => {
      const data = s.processAsyncData[processId];
      data.maxDepth = Math.max(data.maxDepth, reply.tracks.length);
      const section = data.sections[sectionNumber];
      section.tracks = reply.tracks;
      section.state = LODState.Loaded;
      addScopes(reply.scopes);
    });
  };

  const setProcessSection = (processId: string, iSection: number) => {
    updateState((s) => {
      s.processAsyncData[processId].sections[iSection] = {
        sectionSequenceNumber: iSection,
        sectionLod: 0,
        state: LODState.Requested,
        tracks: [],
      };
    });
  };

  const addProcess = (process: Process) => {
    updateState((s) => {
      s.processes.push(process);
    });
  };

  const addThread = (stream: Stream) => {
    updateState((s) => {
      s.threads[stream.streamId] = {
        streamInfo: stream,
        maxDepth: 0,
        minMs: Infinity,
        maxMs: -Infinity,
        block_ids: [],
      };
    });
  };

  const addScopes = (scopes: { [key: number]: ScopeDesc }) => {
    updateState((s) => {
      s.scopes = { ...s.scopes, ...scopes };
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
    addProcessAsyncBlock,
    addBlock,
    addProcess,
    addThread,
    addScopes,
    addBlockData,
    addAsyncData,
    addAsyncBlockData,
    set,
    setProcessSection,
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
  };
}
