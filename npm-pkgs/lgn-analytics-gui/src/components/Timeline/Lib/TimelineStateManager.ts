/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { get } from "svelte/store";

import type { BlockSpansReply } from "@lgn/proto-telemetry/dist/analytics";
import type { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
import type { Process } from "@lgn/proto-telemetry/dist/process";
import type { Stream } from "@lgn/proto-telemetry/dist/stream";

import { loadPromise, loadWrap } from "@/lib/Misc/LoadingStore";
import { makeGrpcClient } from "@/lib/client";
import {
  computePreferredBlockLod,
  processMsOffsetToRoot,
  timestampToMs,
} from "@/lib/time";

import { LODState } from "./LodState";
import type { ProcessAsyncData } from "./ProcessAsyncData";
import type { ThreadBlock } from "./ThreadBlock";
import { TimelineState } from "./TimelineState";
import type { TimelineStateStore } from "./TimelineStateStore";
import { createTimelineStateStore } from "./TimelineStateStore";

const MAX_NB_REQUEST_IN_FLIGHT = 16;

export class TimelineStateManager {
  state: TimelineStateStore;
  process: Process | undefined = undefined;
  rootStartTime = NaN;
  private client: PerformanceAnalyticsClientImpl;
  private processId: string;
  private nbRequestsInFlight = 0;
  constructor(
    processId: string,
    canvasWidth: number,
    start: number | null,
    end: number | null
  ) {
    this.client = makeGrpcClient();
    this.processId = processId;
    this.state = createTimelineStateStore(
      new TimelineState(canvasWidth, start, end)
    );
  }

  async init() {
    this.process = (
      await this.client.find_process({
        processId: this.processId,
      })
    ).process;
    if (!this.process) {
      throw new Error(`Process ${this.processId} not found`);
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
    const { streams } = await this.client.list_process_streams({
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
    const { processes } = await this.client.list_process_children({
      processId: process.processId,
    });

    const promises = processes.map((process) => {
      this.state.addProcess(process);
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
    sectionLod: number,
    processId: string
  ) {
    const sectionWidthMs = 1000.0;
    const sectionTimeRange = [
      sectionSequenceNumber * sectionWidthMs,
      (sectionSequenceNumber + 1) * sectionWidthMs,
    ] as [number, number]; //section is in relative ms
    const blocksOfInterest: string[] = [];

    for (const stats of Object.values(processAsyncData.blockStats)) {
      if (this.rangesOverlap(sectionTimeRange, [stats.beginMs, stats.endMs])) {
        blocksOfInterest.push(stats.blockId);
      }
    }

    this.nbRequestsInFlight += 1;
    await loadPromise(
      this.client
        .fetch_async_spans({
          sectionSequenceNumber: sectionSequenceNumber,
          sectionLod: sectionLod,
          blockIds: blocksOfInterest,
        })
        .then(
          (reply) => {
            this.state.addAsyncData(processId, reply, sectionSequenceNumber);
            return this.fetchDynData();
          },
          (e) => {
            console.log("Error in fetch_block_async_spans", e);
            return this.fetchDynData();
          }
        )
        .finally(() => {
          this.nbRequestsInFlight -= 1;
        })
    );
  }

  private async fetchAsyncSpans(process: Process) {
    if (import.meta.env.VITE_LEGION_ANALYTICS_ENABLE_ASYNC_SPANS !== "true") {
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
        this.state.setProcessSection(process.processId, iSection);
        promises.push(
          this.fetchAsyncSpansSection(
            processAsyncData,
            iSection,
            0,
            process.processId
          )
        );
      }
    }
    await Promise.all(promises);
  }

  private async fetchAsyncStats(process: Process) {
    if (import.meta.env.VITE_LEGION_ANALYTICS_ENABLE_ASYNC_SPANS !== "true") {
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
        thread.streamInfo.processId === process.processId;
      if (overlaps && blockStatsMissing && blockBelongsToProcess) {
        if (this.nbRequestsInFlight >= MAX_NB_REQUEST_IN_FLIGHT) {
          break;
        }
        sentRequest = true;
        this.nbRequestsInFlight += 1;
        promises.push(
          loadPromise(
            this.client
              .fetch_block_async_stats({
                process,
                stream: thread.streamInfo,
                blockId: block.blockDefinition.blockId,
              })
              .then(
                (reply) => {
                  this.state.addAsyncBlockData(process.processId, reply);
                  return this.fetchDynData();
                },
                (e) => {
                  console.log("Error in fetch_block_async_stats", e);
                  return this.fetchDynData();
                }
              )
              .finally(() => {
                this.nbRequestsInFlight -= 1;
              })
          )
        );
      }
    }

    await Promise.all(promises);
    return sentRequest;
  }

  private async fetchBlocks(process: Process, stream: Stream) {
    const processOffset = processMsOffsetToRoot(this.process, process);
    const response = await loadWrap(async () => {
      return await this.client.list_stream_blocks({
        streamId: stream.streamId,
      });
    });
    for (const block of response.blocks) {
      const beginMs = processOffset + timestampToMs(process, block.beginTicks);
      const endMs = processOffset + timestampToMs(process, block.endTicks);
      this.state.addBlock(beginMs, endMs, block, stream.streamId);
    }
    this.state.addProcessAsyncBlock(process.processId);
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
      if (this.process) {
        sentRequest = await this.fetchAsyncStats(this.process);
      }
    }
    if (!sentRequest) {
      if (this.process) {
        await this.fetchAsyncSpans(this.process);
      }
    }
  }

  async fetchBlockSpans(block: ThreadBlock, lodToFetch: number) {
    const streamId = block.blockDefinition.streamId;
    const process = get(this.state).findStreamProcess(streamId);
    if (!process) {
      throw new Error(`Process ${streamId} not found`);
    }
    block.lods[lodToFetch].state = LODState.Requested;
    const blockId = block.blockDefinition.blockId;
    this.nbRequestsInFlight += 1;
    await loadPromise(
      this.client
        .block_spans({
          blockId: blockId,
          process,
          stream: get(this.state).threads[streamId].streamInfo,
          lodId: lodToFetch,
        })
        .then(
          (o) => {
            this.onLodReceived(o);
            return this.fetchDynData();
          },
          (e) => {
            console.log("Error fetching block spans", e);
            return this.fetchDynData();
          }
        )
        .finally(() => {
          this.nbRequestsInFlight -= 1;
        })
    );
  }

  private onLodReceived(response: BlockSpansReply) {
    if (!response.lod) {
      throw new Error(`Error fetching spans for block ${response.blockId}`);
    }
    this.state.addBlockData(response);
  }
}
