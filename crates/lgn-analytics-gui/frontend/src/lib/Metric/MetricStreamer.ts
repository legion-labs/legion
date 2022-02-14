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
    this.currentMinMs = Math.min(0, ...get(this.metricStore).map((s) => s.min));
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
    if (lod !== this.lod) {
      this.lod = lod;
      this.fetchSelectedMetricsAsync();
    }
  }

  async fetchSelectedMetricsAsync() {
    let metrics = get(this.metricStore).filter((m) => m.enabled);
    let result = await Promise.all(
      metrics.map(async (m) => {
        let result = await this.client!.fetch_process_metric({
          blocks: Array.from(
            m.getViewportBlocks(this.currentMinMs, this.currentMaxMs)
          ),
          params: {
            lod: this.lod!,
            processId: this.processId,
            tscFrequency: this.tscFrequency,
            processStartTicks: this.processStartTicks,
            metricName: m.name,
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
        let m = metrics.filter((m) => m.name === reply.name)[0];
        if (m) {
          let index = metrics.indexOf(m);
          let metric = metrics[index];
          if (metric.store(reply.result)) {
            metrics[index] = metric;
          }
        }
      });
      return metrics;
    });
  }
}
