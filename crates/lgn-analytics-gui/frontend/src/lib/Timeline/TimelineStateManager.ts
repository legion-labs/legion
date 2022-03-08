/* eslint-disable @typescript-eslint/no-non-null-assertion */
import {
  BlockSpansReply,
  PerformanceAnalyticsClientImpl,
} from "@lgn/proto-telemetry/dist/analytics";
import { Process } from "@lgn/proto-telemetry/dist/process";
import { makeGrpcClient } from "../client";
import log from "@lgn/web-client/src/lib/log";
import { LODState, ThreadBlock } from "./ThreadBlock";
import {
  computePreferredBlockLod,
  processMsOffsetToRoot,
  timestampToMs,
} from "../time";
import { Stream } from "@lgn/proto-telemetry/dist/stream";
import { TimelineStateStore } from "./TimelineStateStore";
import { TimelineState } from "./TimelineState";
import Semaphore from "semaphore-async-await";
import { loadWrap as loadWrapAsync } from "../Misc/LoadingStore";

export class TimelineStateManager {
  state: TimelineStateStore;
  process: Process | undefined = undefined;
  rootStartTime = NaN;
  private client: PerformanceAnalyticsClientImpl | null = null;
  private processId: string;
  private semaphore: Semaphore;
  constructor(processId: string) {
    this.processId = processId;
    this.semaphore = new Semaphore(16);
    this.state = new TimelineStateStore(
      new TimelineState(undefined, undefined)
    );
  }

  async initAsync(pixelWidth: number) {
    this.client = await makeGrpcClient();
    this.process = (
      await this.client.find_process({
        processId: this.processId,
      })
    ).process;
    if (!this.process) {
      throw new Error(`Process ${this.processId} not found`);
    }
    this.rootStartTime = Date.parse(this.process.startTime);
    this.state.update((s) => {
      if (this.process) {
        s.processes.push(this.process);
      }
      return s;
    });
    await this.fetchStreamsAsync(this.process);
    await this.fetchChildrenAsync(this.process);
    await this.fetchAsyncSpans(this.process);
    await this.fetchLodsAsync(pixelWidth);
  }

  async fetchStreamsAsync(process: Process) {
    if (!this.client) {
      log.error("no client in fetchStreams");
      return;
    }

    const { streams } = await this.client.list_process_streams({
      processId: process.processId,
    });

    const promises: Promise<void>[] = [];

    streams.forEach((stream) => {
      if (!stream.tags.includes("cpu")) {
        return;
      }

      this.state.update((state) => {
        state.threads[stream.streamId] = {
          streamInfo: stream,
          maxDepth: 0,
          minMs: Infinity,
          maxMs: -Infinity,
          block_ids: [],
        };
        return state;
      });

      promises.push(this.fetchBlocksAsync(process, stream));
    });
    await Promise.all(promises);
  }

  async fetchChildrenAsync(process: Process) {
    if (!this.client) {
      log.error("no client in fetchChildren");
      return;
    }
    const { processes } = await this.client.list_process_children({
      processId: process.processId,
    });

    // commented-out - children will be collapsed by default
    // we should really fetch all the descendents server-side to accomplish this in fewer queries
    // for (let i = 0; i < processes.length; ++i) {
    //   await fetchChildren(processes[i]);
    // }

    const promises = processes.map((process) => {
      this.state.update((s) => {
        s.processes.push(process);
        return s;
      });
      return this.fetchStreamsAsync(process);
    });
    await Promise.all(promises);
  }

  private rangesOverlap(
    range1: [number, number],
    range2: [number, number]
  ): boolean {
    return range1[0] <= range2[1] && range2[0] <= range1[1];
  }

  private async fetchAsyncSpans(process: Process) {
    if (!this.client) {
      log.error("no client in fetchAsyncSpans");
      return;
    }
    const section = [0.0, 1000.0] as [number, number]; //section is in relative ms

    const blocksOfInterest: string[] = [];
    for (const streamId in this.state.value.threads) {
      const thread = this.state.value.threads[streamId];
      if (thread.streamInfo.processId === process.processId) {
        thread.block_ids.forEach((block_id) => {
          const stats = this.state.value.blocks[block_id].asyncStats;
          if (this.rangesOverlap(section, [stats!.beginMs, stats!.endMs])) {
            blocksOfInterest.push(block_id);
          }
        });
      }
    }

    const reply = await loadWrapAsync(
      async () =>
        await this.client!.fetch_async_spans({
          sectionSequenceNumber: 1,
          sectionLod: 0,
          blockIds: blocksOfInterest,
        })
    );
    console.log(reply);
  }

  private async fetchBlocksAsync(process: Process, stream: Stream) {
    if (!this.client) {
      log.error("no client in fetchBlocks");
      return;
    }
    const processOffset = processMsOffsetToRoot(this.process, process);
    const response = await loadWrapAsync(async () => {
      return await this.client!.list_stream_blocks({
        streamId: stream.streamId,
      });
    });
    for (const block of response.blocks) {
      const beginMs = processOffset + timestampToMs(process, block.beginTicks);
      const endMs = processOffset + timestampToMs(process, block.endTicks);
      this.state.update((s) => {
        s.minMs = Math.min(s.minMs, beginMs);
        s.maxMs = Math.max(s.maxMs, endMs);
        s.eventCount += block.nbObjects;
        return s;
      });
      const asyncStatsReply = await loadWrapAsync(async () => {
        return await this.client!.fetch_block_async_stats({
          process,
          stream,
          blockId: block.blockId,
        });
      });
      this.state.update((s) => {
        s.threads[stream.streamId].block_ids.push(block.blockId);
        return s;
      });
      this.state.update((s) => {
        s.blocks[block.blockId] = {
          blockDefinition: block,
          beginMs: beginMs,
          endMs: endMs,
          lods: [],
          asyncStats: asyncStatsReply,
        };
        return s;
      });
    }
  }

  async fetchLodsAsync(pixelWidth: number) {
    const range = this.state.value.getViewRange();
    const promises: Promise<void>[] = [];
    for (const block of Object.values(this.state.value.blocks)) {
      const lod = computePreferredBlockLod(pixelWidth, range, block);
      if (lod && !block.lods[lod]) {
        block.lods[lod] = {
          state: LODState.Missing,
          tracks: [],
          lodId: lod,
        };
        promises.push(this.fetchBlockSpansAsync(block, lod));
      }
    }
    await Promise.all(promises);
  }

  async fetchBlockSpansAsync(block: ThreadBlock, lodToFetch: number) {
    if (!this.client) {
      log.error("no client in fetchBlockSpans");
      return;
    }
    const streamId = block.blockDefinition.streamId;
    const process = this.state.value.findStreamProcess(streamId);
    if (!process) {
      throw new Error(`Process ${streamId} not found`);
    }
    block.lods[lodToFetch].state = LODState.Requested;
    const blockId = block.blockDefinition.blockId;
    await loadWrapAsync(async () => {
      await this.semaphore.acquire();
      try {
        await this.client!.block_spans({
          blockId: blockId,
          process,
          stream: this.state.value.threads[streamId].streamInfo,
          lodId: lodToFetch,
        }).then(
          (o) => this.onLodReceived(o),
          (e) => {
            console.log("Error fetching block spans", e);
          }
        );
      } finally {
        this.semaphore.release();
      }
    });
  }

  private onLodReceived(response: BlockSpansReply) {
    if (!response.lod) {
      throw new Error(`Error fetching spans for block ${response.blockId}`);
    }
    const blockLod = response.lod;
    this.state.update((s) => {
      s.ready = true;
      s.scopes = { ...s.scopes, ...response.scopes };
      const block = s.blocks[response.blockId];
      const thread = s.threads[block.blockDefinition.streamId];
      thread.maxDepth = Math.max(thread.maxDepth, blockLod.tracks.length);
      thread.minMs = Math.min(thread.minMs, response.beginMs);
      thread.maxMs = Math.max(thread.maxMs, response.endMs);
      thread.block_ids.push(response.blockId);
      block.lods[blockLod.lodId].tracks = blockLod.tracks;
      return s;
    });
  }
}
