/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { get } from "svelte/store";
import {
  BlockSpansReply,
  BlockAsyncEventsStatReply,
  PerformanceAnalyticsClientImpl,
  SpanTrack,
} from "@lgn/proto-telemetry/dist/analytics";
import { Process } from "@lgn/proto-telemetry/dist/process";
import { makeGrpcClient } from "../client";
import log from "@lgn/web-client/src/lib/log";
import { ThreadBlock } from "./ThreadBlock";
import { LODState } from "./LodState";
import { AsyncSection } from "./AsyncSection";
import {
  computePreferredBlockLod,
  processMsOffsetToRoot,
  timestampToMs,
} from "../time";
import { Stream } from "@lgn/proto-telemetry/dist/stream";
import type { TimelineStateStore } from "./TimelineStateStore";
import { createTimelineStateStore } from "./TimelineStateStore";
import { TimelineState } from "./TimelineState";
import { ProcessAsyncData } from "./ProcessAsyncData";
import Semaphore from "semaphore-async-await";
import { loadWrap } from "../Misc/LoadingStore";

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
    this.state = createTimelineStateStore(
      new TimelineState(undefined, undefined)
    );
  }

  async init(pixelWidth: number) {
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
    await this.fetchStreams(this.process);
    await this.fetchChildren(this.process);
    await this.fetchAsyncSpans(this.process);
    await this.fetchLods(pixelWidth);
  }

  async fetchStreams(process: Process) {
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

      promises.push(this.fetchBlocks(process, stream));
    });
    await Promise.all(promises);
  }

  async fetchChildren(process: Process) {
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
      return this.fetchStreams(process);
    });
    await Promise.all(promises);
  }

  private rangesOverlap(
    range1: [number, number],
    range2: [number, number]
  ): boolean {
    return range1[0] <= range2[1] && range2[0] <= range1[1];
  }

  private async fetchAsyncSpansSection(
    processAsyncData: ProcessAsyncData,
    sectionSequenceNumber: number,
    sectionLod: number
  ) {
    const sectionWidthMs = 1000.0;
    const sectionTimeRange = [
      sectionSequenceNumber * sectionWidthMs,
      (sectionSequenceNumber + 1) * sectionWidthMs,
    ] as [number, number]; //section is in relative ms
    const blocksOfInterest: string[] = [];
    processAsyncData.blockStats.forEach((stats) => {
      if (
        this.rangesOverlap(sectionTimeRange, [stats!.beginMs, stats!.endMs])
      ) {
        blocksOfInterest.push(stats.blockId);
      }
    });

    const asyncSection = {
      sectionSequenceNumber,
      sectionLod,
      state: LODState.Requested,
      tracks: [] as SpanTrack[],
    };
    processAsyncData.sections.push(asyncSection);

    const reply = await loadWrap(
      async () =>
        await this.client!.fetch_async_spans({
          sectionSequenceNumber,
          sectionLod,
          blockIds: blocksOfInterest,
        })
    );
    const nbTracks = reply.tracks.length;
    processAsyncData.maxDepth = Math.max(processAsyncData.maxDepth, nbTracks);
    asyncSection.tracks = reply.tracks;
    asyncSection.state = LODState.Loaded;

    this.state.update((s) => {
      s.scopes = { ...s.scopes, ...reply.scopes };
      return s;
    });
  }

  private async fetchAsyncSpans(process: Process) {
    if (!this.client) {
      log.error("no client in fetchAsyncSpans");
      return;
    }
    const processAsyncData = get(this.state).processAsyncData[
      process.processId
    ];

    const sectionWidthMs = 1000.0;
    const firstSection = Math.floor(processAsyncData.minMs / sectionWidthMs);
    const lastSection = Math.floor(processAsyncData.maxMs / sectionWidthMs);
    const promises: Promise<void>[] = [];
    for (let iSection = firstSection; iSection <= lastSection; iSection += 1) {
      promises.push( this.fetchAsyncSpansSection(processAsyncData, iSection, 0) );
    }
    await Promise.all(promises);
  }

  private async fetchBlocks(process: Process, stream: Stream) {
    if (!this.client) {
      log.error("no client in fetchBlocks");
      return;
    }
    const blockStats: BlockAsyncEventsStatReply[] = [];
    const asyncSections: AsyncSection[] = [];
    const asyncData = {
      processId: process.processId,
      maxDepth: 0,
      minMs: Infinity,
      maxMs: -Infinity,
      blockStats,
      sections: asyncSections,
    };
    const processOffset = processMsOffsetToRoot(this.process, process);
    const response = await loadWrap(async () => {
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
      const asyncStatsReply = await loadWrap(async () => {
        return await this.client!.fetch_block_async_stats({
          process,
          stream,
          blockId: block.blockId,
        });
      });
      asyncData.minMs = Math.min(asyncData.minMs, asyncStatsReply.beginMs);
      asyncData.maxMs = Math.max(asyncData.maxMs, asyncStatsReply.endMs);
      blockStats.push(asyncStatsReply);
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
        };
        return s;
      });
    }
    get(this.state).processAsyncData[process.processId] = asyncData;
  }

  async fetchLods(pixelWidth: number) {
    const range = get(this.state).getViewRange();
    const promises: Promise<void>[] = [];
    for (const block of Object.values(get(this.state).blocks)) {
      const lod = computePreferredBlockLod(pixelWidth, range, block);
      if (lod && !block.lods[lod]) {
        block.lods[lod] = {
          state: LODState.Missing,
          tracks: [],
          lodId: lod,
        };
        promises.push(this.fetchBlockSpans(block, lod));
      }
    }
    await Promise.all(promises);
  }

  async fetchBlockSpans(block: ThreadBlock, lodToFetch: number) {
    if (!this.client) {
      log.error("no client in fetchBlockSpans");
      return;
    }
    const streamId = block.blockDefinition.streamId;
    const process = get(this.state).findStreamProcess(streamId);
    if (!process) {
      throw new Error(`Process ${streamId} not found`);
    }
    block.lods[lodToFetch].state = LODState.Requested;
    const blockId = block.blockDefinition.blockId;
    await loadWrap(async () => {
      await this.semaphore.acquire();
      try {
        await this.client!.block_spans({
          blockId: blockId,
          process,
          stream: get(this.state).threads[streamId].streamInfo,
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
