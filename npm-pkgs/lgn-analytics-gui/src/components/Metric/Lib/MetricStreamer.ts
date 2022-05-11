import Semaphore from "semaphore-async-await";
import { get } from "svelte/store";

import type { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";

import { MetricState } from "./MetricState";
import { getMetricStore } from "./MetricStore";
import type { MetricStore } from "./MetricStore";

export class MetricStreamer {
  currentMinMs = -Infinity;
  currentMaxMs = Infinity;
  metricStore: MetricStore;

  #semaphore: Semaphore;
  #processId: string;
  #client: PerformanceAnalyticsClientImpl;

  constructor(client: PerformanceAnalyticsClientImpl, processId: string) {
    this.#processId = processId;
    this.#client = client;
    this.#semaphore = new Semaphore(8);

    this.metricStore = getMetricStore();
  }

  async initialize() {
    const blocks = (
      await this.#client.list_process_blocks({
        processId: this.#processId,
        tag: "metrics",
      })
    ).blocks;

    const blockManifests = await Promise.all(
      blocks.map(async (block) => {
        const blockManifest = await this.#client?.fetch_block_metric_manifest({
          blockId: block.blockId,
          streamId: block.streamId,
          processId: this.#processId,
        });
        return blockManifest;
      })
    );

    const metricStates = new Map<string, MetricState>();

    for (const blockManifest of blockManifests) {
      for (const metricDesc of blockManifest.metrics) {
        if (!metricStates.get(metricDesc.name)) {
          metricStates.set(metricDesc.name, new MetricState(metricDesc));
        }
        const metricState = metricStates.get(metricDesc.name);
        metricState?.registerBlock(blockManifest);
      }
    }

    this.metricStore.registerMetrics(Array.from(metricStates.values()));
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
      // eslint-disable-next-line @typescript-eslint/no-misused-promises
      metric.blocks.forEach(async (block) => {
        await this.#semaphore.acquire();
        try {
          const blockData = await this.#client.fetch_block_metric({
            processId: this.#processId,
            streamId: block.streamId,
            metricName: metric.name,
            blockId: block.blockId,
            lod: lod,
          });
          // TODO: Is this really correct? It seems the value is guaranteed not to be undefined
          if (blockData !== undefined) {
            this.metricStore.registerBlock(
              blockData,
              block.blockId,
              metric.name
            );
          }
        } finally {
          this.#semaphore.release();
        }
      });
    });
  }
}
