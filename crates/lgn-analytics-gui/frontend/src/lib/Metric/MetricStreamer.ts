import { makeGrpcClient } from "@/lib/client";
import { PerformanceAnalyticsClientImpl } from "@lgn/proto-telemetry/dist/analytics";
import { get, Writable, writable } from "svelte/store";
import { MetricState } from "./MetricState";

export class MetricStreamer {
  currentMinMs: number = -Infinity;
  currentMaxMs: number = Infinity;
  metricStore: Writable<MetricState[]>;
  private client: PerformanceAnalyticsClientImpl | null = null;
  private lod: number | null = null;
  private processId: string;
  private tscFrequency: number = 0;
  private processStartTicks: number = 0;
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

  switchMetricFlag(
    metricState: MetricState,
    e: MouseEvent & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    this.metricStore.update((data) => {
      let index = data.indexOf(metricState);
      let metric = data[index];
      metric.enabled = e.currentTarget.checked;
      data[index] = metric;
      return data;
    });
  }

  tick(lod: number, min: number, max: number) {
    this.currentMinMs = min;
    this.currentMaxMs = max;
    this.lod = lod;
    this.fetchSelectedMetricsAsync(lod);
  }

  async fetchSelectedMetricsAsync(lod: number) {
    let metrics = get(this.metricStore).filter((m) => m.enabled);

    const missingBlocks = metrics.map((m) => {
      return {
        name: m.name,
        blocks: Array.from(
          m.getMissingBlocks(this.currentMinMs, this.currentMaxMs, lod)
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

    let result = await Promise.all(
      metrics.map(async (m) => {
        let result = await this.client!.fetch_process_metric({
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
        let metric = metrics.filter((m) => m.name === reply.name)[0];
        if (metric) {
          let index = metrics.indexOf(metric);
          let metricInArray = metrics[index];
          if (metricInArray.store(reply.result)) {
            metrics[index] = metricInArray;
          }
        }
      });
      return metrics;
    });
  }
}
