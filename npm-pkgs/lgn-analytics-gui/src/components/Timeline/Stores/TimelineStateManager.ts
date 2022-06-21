/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { get } from "svelte/store";

import type { BlockSpansReply } from "@lgn/proto-telemetry/dist/analytics";
import type { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
import type { Process } from "@lgn/proto-telemetry/dist/process";
import type { Stream } from "@lgn/proto-telemetry/dist/stream";
import { displayError } from "@lgn/web-client/src/lib/errors";
import log from "@lgn/web-client/src/lib/log";

import { loadPromise, loadWrap } from "@/lib/Misc/LoadingStore";
import {
  computePreferredBlockLod,
  processMsOffsetToRoot,
  timestampToMs,
} from "@/lib/time";

import { LODState } from "../Lib/LodState";
import type { ThreadBlock } from "../Lib/ThreadBlock";
import { TimelineState } from "./TimelineState";
import type { TimelineStateStore } from "./TimelineStateStore";
import { createTimelineStateStore } from "./TimelineStateStore";

const MAX_NB_REQUEST_IN_FLIGHT = 16;

export class TimelineStateManager {
  state: TimelineStateStore;
  process: Process | undefined = undefined;
  rootStartTime = NaN;

  #client: PerformanceAnalyticsClientImpl;
  #processId: string;
  #nbRequestsInFlight = 0;

  constructor(
    client: PerformanceAnalyticsClientImpl,
    processId: string,
    canvasWidth: number,
    start: number | null,
    end: number | null
  ) {
    this.#client = client;
    this.#processId = processId;
    this.state = createTimelineStateStore(
      new TimelineState(canvasWidth, start, end)
    );
  }

  async init() {
    this.process = (
      await this.#client.find_process({
        processId: this.#processId,
      })
    ).process;
    if (!this.process) {
      throw new Error(`Process ${this.#processId} not found`);
    }
    this.rootStartTime = Date.parse(this.process.startTime);
    this.state.addProcess(this.process);
    await this.fetchStreams(this.process);
    this.initViewRange(this.process);
    await this.fetchChildren(this.process);
    await this.fetchDynData();
  }

  private initViewRange(process: Process) {
    const state = get(this.state);

    if (state.createdWithParameters()) {
      return;
    }

    const blocks: ThreadBlock[] = [];
    for (const block of Object.values(state.blocks)) {
      const streamId = block.blockDefinition.streamId;
      const thread = state.threads[streamId];
      if (thread.streamInfo.processId === process.processId) {
        blocks.push(block);
      }
    }
    blocks.sort((a, b) => (a.endMs > b.endMs ? -1 : 1));
    let nbEvents = 0;
    for (let i = 0; i < blocks.length; i += 1) {
      nbEvents += blocks[i].blockDefinition.nbObjects;
      this.state.setViewRange([blocks[i].beginMs, blocks[0].endMs]);
      if (nbEvents > 10000) {
        return;
      }
    }
  }

  async fetchStreams(process: Process) {
    const { streams } = await this.#client.list_process_streams({
      processId: process.processId,
    });

    if (!streams.length) {
      throw new Error(`No streams available in process ${process.processId}.`);
    }

    const promises: Promise<void>[] = [];

    streams.forEach((stream) => {
      if (!stream.tags.includes("cpu")) {
        return;
      }
      this.state.addThread(stream);
      promises.push(this.fetchBlocks(process, stream));
    });

    await Promise.all(promises);
  }

  private async fetchChildren(process: Process) {
    const { processes } = await this.#client.list_process_children({
      processId: process.processId,
    });

    const promises = processes.map((process) => {
      this.state.addProcess(process);
      return this.fetchStreams(process);
    });
    await Promise.all(promises);
  }

  private async fetchBlocks(process: Process, stream: Stream) {
    const processOffset = processMsOffsetToRoot(this.process, process);
    const response = await loadWrap(async () => {
      return await this.#client.list_stream_blocks({
        streamId: stream.streamId,
      });
    });
    for (const block of response.blocks) {
      const beginMs = processOffset + timestampToMs(process, block.beginTicks);
      const endMs = processOffset + timestampToMs(process, block.endTicks);
      this.state.addBlock(beginMs, endMs, block, stream.streamId);
    }
  }

  async fetchThreadData(): Promise<boolean> {
    const state = get(this.state);
    const range = state.viewRange;
    const promises: Promise<void>[] = [];
    let sentRequest = false;
    for (const block of Object.values(state.blocks)) {
      const lod = computePreferredBlockLod(state.canvasWidth, range, block);
      if (lod === null) {
        continue;
      }

      let lodInfo;
      if (lod in block.lods) {
        lodInfo = block.lods[lod];
      } else {
        lodInfo = {
          state: LODState.Missing,
          tracks: [],
          lodId: lod,
        };
        block.lods[lod] = lodInfo;
      }
      if (lodInfo.state === LODState.Missing) {
        sentRequest = true;
        promises.push(this.fetchBlockSpans(block, lod));
      }

      if (this.#nbRequestsInFlight >= MAX_NB_REQUEST_IN_FLIGHT) {
        break;
      }
    }
    await Promise.all(promises);
    return sentRequest;
  }

  async fetchDynData() {
    await this.fetchThreadData();
  }

  async fetchBlockSpans(block: ThreadBlock, lodToFetch: number) {
    const streamId = block.blockDefinition.streamId;
    const process = get(this.state).findStreamProcess(streamId);
    if (!process) {
      throw new Error(`Process ${streamId} not found`);
    }
    block.lods[lodToFetch].state = LODState.Requested;
    const blockId = block.blockDefinition.blockId;
    this.#nbRequestsInFlight += 1;
    await loadPromise(
      this.#client
        .block_spans({
          blockId: blockId,
          process,
          stream: get(this.state).threads[streamId].streamInfo,
          lodId: lodToFetch,
        })
        .then(
          (o) => {
            this.#nbRequestsInFlight -= 1;
            this.onLodReceived(o);
            return this.fetchDynData();
          },
          (error) => {
            log.error(`Error fetching block spans: ${displayError(error)}`);
            this.#nbRequestsInFlight -= 1;
            return this.fetchDynData();
          }
        )
    );
  }

  private onLodReceived(response: BlockSpansReply) {
    if (!response.lod) {
      throw new Error(`Error fetching spans for block ${response.blockId}`);
    }
    this.state.addBlockData(response);
  }
}
