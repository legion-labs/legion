import { MetricSelectionState } from "@/components/Metric/MetricSelectionState";
import { makeGrpcClient } from "@/lib/client";
import { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
import Semaphore from "semaphore-async-await";
import { get, Writable, writable } from "svelte/store";
import { MetricState } from "./MetricState";

export class MetricStreamer {
  currentMinMs = -Infinity;
  currentMaxMs = Infinity;
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
    const blocks = (
      await this.client.list_process_blocks({
        processId: this.processId,
        tag: "metrics",
      })
    ).blocks;

    const blockManifests = await Promise.all(
      blocks.map(async (block) => {
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
            metricStates.set(metricDesc.name, new MetricState(metricDesc));
          }
          const metricState = metricStates.get(metricDesc.name);
          metricState?.registerBlock(blockManifest);
        }
      }
    }

    this.metricStore.set(Array.from(metricStates.values()));
    this.currentMinMs = Math.min(...get(this.metricStore).map((s) => s.min));
    this.currentMaxMs = Math.max(...get(this.metricStore).map((s) => s.max));
  }

  tick(lod: number, min: number, max: number) {
    this.currentMinMs = min;
    this.currentMaxMs = max;
    this.fetchSelectedMetrics(lod);
  }

  fetchSelectedMetrics(lod: number) {
    const metrics = get(this.metricStore).filter((m) => m.canBeDisplayed());

    const missingBlocks = metrics.map((m) => {
      return {
        name: m.name,
        blocks: Array.from(
          m.requestMissingBlocks(this.currentMinMs, this.currentMaxMs, lod)
        ),
      };
    });

    if (!missingBlocks.flatMap((b) => b.blocks).length) {
      return;
    }

    missingBlocks.forEach((metric) => {
      metric.blocks.forEach(async (block) => {
        await this.semaphore.acquire();
        try {
          const blockData = await this.client?.fetch_block_metric({
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
