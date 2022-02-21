import { MetricSelectionState } from "@/components/Metric/MetricSelectionState";
import { makeGrpcClient } from "@/lib/client";
import { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
import { get, Writable, writable } from "svelte/store";
import { MetricState } from "./MetricState";

export class MetricStreamer {
  currentMinMs = -Infinity;
  currentMaxMs = Infinity;
  metricStore: Writable<MetricState[]>;
  private client: PerformanceAnalyticsClientImpl | null = null;
  private processId: string;
  private tscFrequency = 0;
  private processStartTicks = 0;
  constructor(processId: string) {
    this.processId = processId;
    this.metricStore = writable([]);
  }

  async initializeAsync() {
    this.client = await makeGrpcClient();
    const reply = await this.client.list_process_metrics({
      processId: this.processId,
    });
    this.tscFrequency = reply.tscFrequency;
    this.processStartTicks = reply.processStartTicks;
    this.metricStore.set(reply.metrics.map((m) => new MetricState(true, m)));
    this.currentMinMs = Math.min(...get(this.metricStore).map((s) => s.min));
    this.currentMaxMs = Math.max(...get(this.metricStore).map((s) => s.max));
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

  tick(lod: number, min: number, max: number) {
    this.currentMinMs = min;
    this.currentMaxMs = max;
    this.fetchSelectedMetricsAsync(lod);
  }

  async fetchSelectedMetricsAsync(lod: number) {
    const metrics = get(this.metricStore).filter((m) => m.enabled);

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

    console.log(
      `Fetching \n${missingBlocks
        .flatMap((b) => b.blocks)
        .map((b) => `${b.blockId} (${lod})`)
        .join("\n")}`
    );

    const result = await Promise.all(
      metrics.map(async (m) => {
        const result = await this.client?.fetch_process_metric({
          blocks: Array.from(
            m.getViewportBlocks(this.currentMinMs, this.currentMaxMs)
          ),
          params: {
            lod: lod,
            metricName: m.name,
            processId: this.processId,
            tscFrequency: this.tscFrequency,
            processStartTicks: this.processStartTicks,
          },
        });
        return {
          result: result,
          name: m.name,
        };
      })
    );

    this.metricStore.update((metrics) => {
      result.forEach((reply) => {
        const metric = metrics.filter((m) => m.name === reply.name)[0];
        if (metric) {
          const index = metrics.indexOf(metric);
          const metricInArray = metrics[index];
          if (reply.result) {
            if (metricInArray.store(reply.result)) {
              metrics[index] = metricInArray;
            }
          }
        }
      });
      return metrics;
    });
  }
}
