import { MetricSelectionState } from "@/components/Metric/MetricSelectionState";
import { makeGrpcClient } from "@/lib/client";
import { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
import Semaphore from "semaphore-async-await";
import { get, Writable, writable } from "svelte/store";
import { MetricState } from "./MetricState";

export class MetricStreamer {
  minTick = NaN;
  maxTick = NaN;
  tscFrequency = NaN;
  processStartTicks = NaN;
  metricStore: Writable<MetricState[]>;
  private client: PerformanceAnalyticsClientImpl | null = null;
  private processId: string;
  private semaphore: Semaphore;
  constructor(processId: string) {
    this.processId = processId;
    this.metricStore = writable([]);
    this.semaphore = new Semaphore(8);
  }

  async initializeAsync() {
    this.client = await makeGrpcClient();
    const processBlockReply = await this.client.list_process_blocks({
      processId: this.processId,
      tag: "metrics",
    });

    this.tscFrequency = processBlockReply.tscFrequency;
    this.processStartTicks = processBlockReply.processStartTicks;

    const blockManifests = await Promise.all(
      processBlockReply.blocks.map(async (block) => {
        const blockManifest = await this.client?.fetch_block_metric_manifest({
          blockId: block.blockId,
          streamId: block.streamId,
          processId: this.processId,
        });
        return blockManifest;
      })
    );

    const metricStates = new Map<string, MetricState>();

    for (const blockManifest of blockManifests) {
      if (blockManifest) {
        for (const metricDesc of blockManifest?.metrics) {
          if (!metricStates.get(metricDesc.name)) {
            metricStates.set(
              metricDesc.name,
              new MetricState(true, metricDesc, this.processStartTicks)
            );
          }
          const metricState = metricStates.get(metricDesc.name);
          metricState?.registerBlock(blockManifest);
        }
      }
    }

    this.metricStore.set(Array.from(metricStates.values()));
    this.minTick = Math.min(...get(this.metricStore).map((s) => s.minTick));
    this.maxTick = Math.max(...get(this.metricStore).map((s) => s.maxTick));
  }

  getTickOffsetMs(tick: number) {
    return ((tick - this.processStartTicks) * 1_000) / this.tscFrequency;
  }

  getTickRawMs(tick: number) {
    return (tick * 1000) / this.tscFrequency;
  }

  updateFromSelectionState(metricSelectionState: MetricSelectionState) {
    this.metricStore.update((data) => {
      const metric = data.filter(
        (m) => m.name === metricSelectionState.name
      )[0];
      if (metric) {
        metric.enabled = metricSelectionState.selected;
        const index = data.indexOf(metric);
        data[index] = metric;
      }
      return data;
    });
  }

  tick(lod: number, minTick: number, maxTick: number) {
    this.fetchSelectedMetrics(lod, minTick, maxTick);
  }

  fetchSelectedMetrics(lod: number, minTick: number, maxTick: number) {
    const metrics = get(this.metricStore).filter((m) => m.enabled);

    const missingBlocks = metrics.map((m) => {
      return {
        name: m.name,
        blocks: Array.from(m.requestMissingBlocks(minTick, maxTick, lod)),
      };
    });

    if (!missingBlocks.flatMap((b) => b.blocks).length) {
      return;
    }

    console.log(
      `Fetching \n${missingBlocks
        .flatMap((b) => b.blocks)
        .map((b) => `${b.blockId} (${lod})`)
        .join("\n")}`
    );

    missingBlocks.forEach((metric) => {
      metric.blocks.forEach(async (block) => {
        await this.semaphore.acquire();
        try {
          const blockData = await this.client?.fetch_block_metric({
            processStartTicks: this.processStartTicks,
            tscFrequency: this.tscFrequency,
            processId: this.processId,
            streamId: block.streamId,
            metricName: metric.name,
            blockId: block.blockId,
            lod: lod,
          });
          this.metricStore.update((metrics) => {
            const m = metrics.filter((m) => m.name === metric.name)[0];
            if (m) {
              const index = metrics.indexOf(m);
              if (blockData && m.store(block.blockId, blockData)) {
                metrics[index] = m;
              }
            }
            return metrics;
          });
        } finally {
          this.semaphore.release();
        }
      });
    });
  }
}
