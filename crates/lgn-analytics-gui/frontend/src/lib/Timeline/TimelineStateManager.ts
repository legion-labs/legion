/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { get } from "svelte/store";
import type { BlockSpansReply } from "@lgn/proto-telemetry/dist/analytics";
import type { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
import type { Process } from "@lgn/proto-telemetry/dist/process";
import { makeGrpcClient } from "../client";
import log from "@lgn/web-client/src/lib/log";
import type { ThreadBlock } from "./ThreadBlock";
import { LODState } from "./LodState";
import type { AsyncSection } from "./AsyncSection";
import {
  computePreferredBlockLod,
  processMsOffsetToRoot,
  timestampToMs,
} from "../time";
import type { Stream } from "@lgn/proto-telemetry/dist/stream";
import type { TimelineStateStore } from "./TimelineStateStore";
import { createTimelineStateStore } from "./TimelineStateStore";
import { TimelineState } from "./TimelineState";
import type { ProcessAsyncData } from "./ProcessAsyncData";
import { loadPromise, loadWrap } from "../Misc/LoadingStore";

const MAX_NB_REQUEST_IN_FLIGHT = 16;

export class TimelineStateManager {
  state: TimelineStateStore;
  process: Process | undefined = undefined;
  rootStartTime = NaN;
  private client: PerformanceAnalyticsClientImpl | null = null;
  private processId: string;
  private nbRequestsInFlight = 0;
  constructor(
    processId: string,
    canvasWidth: number,
    start: number | null,
    end: number | null
  ) {
    this.processId = processId;
    this.state = createTimelineStateStore(
      new TimelineState(canvasWidth, start, end)
    );
  }

  async init() {
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
    this.initViewRange(this.process);
    await this.fetchChildren(this.process);
    await this.fetchDynData();
  }

  initViewRange(process: Process) {
    const blocks: ThreadBlock[] = [];
    const state = get(this.state);
    for (const block of Object.values(state.blocks)) {
      const streamId = block.blockDefinition.streamId;
      const thread = state.threads[streamId];
      if (thread.streamInfo.processId == process.processId) {
        blocks.push(block);
      }
    }
    blocks.sort((a, b) => (a.endMs > b.endMs ? -1 : 1));
    let nbEvents = 0;
    for (let i = 0; i < blocks.length; i += 1) {
      nbEvents += blocks[i].blockDefinition.nbObjects;
      if (nbEvents > 10000) {
        this.state.update((s) => {
          s.setViewRange([blocks[i].beginMs, blocks[0].endMs]);
          return s;
        });
        return;
      }
    }
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
    section: AsyncSection
  ) {
    const sectionWidthMs = 1000.0;
    const sectionTimeRange = [
      section.sectionSequenceNumber * sectionWidthMs,
      (section.sectionSequenceNumber + 1) * sectionWidthMs,
    ] as [number, number]; //section is in relative ms
    const blocksOfInterest: string[] = [];

    for (const stats of Object.values(processAsyncData.blockStats)) {
      if (
        this.rangesOverlap(sectionTimeRange, [stats!.beginMs, stats!.endMs])
      ) {
        blocksOfInterest.push(stats.blockId);
      }
    }

    this.nbRequestsInFlight += 1;
    await loadPromise(
      this.client!.fetch_async_spans({
        sectionSequenceNumber: section.sectionSequenceNumber,
        sectionLod: section.sectionLod,
        blockIds: blocksOfInterest,
      }).then(
        (reply) => {
          this.nbRequestsInFlight -= 1;
          const nbTracks = reply.tracks.length;
          processAsyncData.maxDepth = Math.max(
            processAsyncData.maxDepth,
            nbTracks
          );
          section.tracks = reply.tracks;
          section.state = LODState.Loaded;
          this.state.update((s) => {
            s.scopes = { ...s.scopes, ...reply.scopes };
            return s;
          });
          return this.fetchDynData();
        },
        (e) => {
          this.nbRequestsInFlight -= 1;
          console.log("Error in fetch_block_async_spans", e);
          return this.fetchDynData();
        }
      )
    );
  }

  private async fetchAsyncSpans(process: Process) {
    if (import.meta.env.VITE_LEGION_ANALYTICS_ENABLE_ASYNC_SPANS != "true") {
      return;
    }

    if (!this.client) {
      log.error("no client in fetchAsyncSpans");
      return;
    }
    const state = get(this.state);
    const viewRange = state.getViewRange();
    const processAsyncData = state.processAsyncData[process.processId];

    const sectionWidthMs = 1000.0;
    const firstSection = Math.floor(viewRange[0] / sectionWidthMs);
    const lastSection = Math.floor(viewRange[1] / sectionWidthMs);
    const promises: Promise<void>[] = [];
    for (let iSection = firstSection; iSection <= lastSection; iSection += 1) {
      if (this.nbRequestsInFlight >= MAX_NB_REQUEST_IN_FLIGHT) {
        break;
      }
      if (!(iSection in processAsyncData.sections)) {
        const section = {
          sectionSequenceNumber: iSection,
          sectionLod: 0,
          state: LODState.Requested,
          tracks: [],
        };
        processAsyncData.sections[iSection] = section;
        promises.push(this.fetchAsyncSpansSection(processAsyncData, section));
      }
    }
    await Promise.all(promises);
  }

  private async fetchAsyncStats(process: Process) {
    if (import.meta.env.VITE_LEGION_ANALYTICS_ENABLE_ASYNC_SPANS != "true") {
      return true;
    }
    const state = get(this.state);
    const asyncData = state.processAsyncData[process.processId];
    const promises: Promise<void>[] = [];
    let sentRequest = false;
    const viewRange = state.getViewRange();
    for (const block of Object.values(state.blocks)) {
      const streamId = block.blockDefinition.streamId;
      const thread = state.threads[streamId];
      const overlaps = this.rangesOverlap(viewRange, [
        block.beginMs,
        block.endMs,
      ]);
      const blockStatsMissing = !(
        block.blockDefinition.blockId in asyncData.blockStats
      );
      const blockBelongsToProcess =
        thread.streamInfo.processId == process.processId;
      if (overlaps && blockStatsMissing && blockBelongsToProcess) {
        if (this.nbRequestsInFlight >= MAX_NB_REQUEST_IN_FLIGHT) {
          break;
        }
        sentRequest = true;
        this.nbRequestsInFlight += 1;
        promises.push(
          loadPromise(
            this.client!.fetch_block_async_stats({
              process,
              stream: thread.streamInfo,
              blockId: block.blockDefinition.blockId,
            }).then(
              (reply) => {
                this.nbRequestsInFlight -= 1;
                asyncData.minMs = Math.min(asyncData.minMs, reply.beginMs);
                asyncData.maxMs = Math.max(asyncData.maxMs, reply.endMs);
                asyncData.blockStats[reply.blockId] = reply;
                return this.fetchDynData();
              },
              (e) => {
                this.nbRequestsInFlight -= 1;
                console.log("Error in fetch_block_async_stats", e);
                return this.fetchDynData();
              }
            )
          )
        );
      }
    }

    await Promise.all(promises);
    return sentRequest;
  }

  private async fetchBlocks(process: Process, stream: Stream) {
    if (!this.client) {
      log.error("no client in fetchBlocks");
      return;
    }
    const asyncSections: AsyncSection[] = [];
    const asyncData = {
      processId: process.processId,
      maxDepth: 0,
      minMs: Infinity,
      maxMs: -Infinity,
      blockStats: {},
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

  async fetchThreadData(): Promise<boolean> {
    const state = get(this.state);
    const range = state.getViewRange();
    const promises: Promise<void>[] = [];
    let sentRequest = false;
    for (const block of Object.values(state.blocks)) {
      const lod = computePreferredBlockLod(state.canvasWidth, range, block);
      if (lod && !block.lods[lod]) {
        block.lods[lod] = {
          state: LODState.Missing,
          tracks: [],
          lodId: lod,
        };
        sentRequest = true;
        promises.push(this.fetchBlockSpans(block, lod));
      }
      if (this.nbRequestsInFlight >= MAX_NB_REQUEST_IN_FLIGHT) {
        break;
      }
    }
    await Promise.all(promises);
    return sentRequest;
  }

  async fetchDynData() {
    let sentRequest = await this.fetchThreadData();
    if (!sentRequest) {
      sentRequest = await this.fetchAsyncStats(this.process!);
    }
    if (!sentRequest) {
      await this.fetchAsyncSpans(this.process!);
    }
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
    this.nbRequestsInFlight += 1;
    await loadPromise(
      this.client!.block_spans({
        blockId: blockId,
        process,
        stream: get(this.state).threads[streamId].streamInfo,
        lodId: lodToFetch,
      }).then(
        (o) => {
          this.nbRequestsInFlight -= 1;
          this.onLodReceived(o);
          return this.fetchDynData();
        },
        (e) => {
          this.nbRequestsInFlight -= 1;
          console.log("Error fetching block spans", e);
          return this.fetchDynData();
        }
      )
    );
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
