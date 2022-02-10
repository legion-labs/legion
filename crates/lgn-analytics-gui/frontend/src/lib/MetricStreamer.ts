import { client } from "@/lib/client";
import { Point } from "@/lib/point";
import {
  MetricDesc,
  ProcessMetricReply,
} from "@lgn/proto-telemetry/dist/metric";
import { Writable, writable } from "svelte/store";

export class MetricState {
  enabled: boolean;
  metricDesc: MetricDesc;
  points: Point[] = [];
  constructor(metricDesc: MetricDesc, enabled: boolean) {
    this.metricDesc = metricDesc;
    this.enabled = enabled;
  }
}

export class MetricStreamer {
  private lod: number;
  private totalMin: number;
  private totalMax: number;
  private processId: string;
  private currentMin: number = -Infinity;
  private currentMax: number = Infinity;
  private metrics: MetricDesc[];
  metricStore: Writable<MetricState[]>;
  constructor(
    processId: string,
    lod: number,
    totalMin: number,
    totalMax: number
  ) {
    this.processId = processId;
    this.lod = lod;
    this.totalMin = totalMin;
    this.totalMax = totalMax;
    this.metrics = [];
    this.metricStore = writable([]);
  }

  async initializeAsync() {
    this.metrics = (
      await client.list_process_metrics({
        processId: this.processId,
      })
    ).metrics;

    this.metricStore.set(this.metrics.map((m) => new MetricState(m, true)));

    await this.fetchSelectedMetricsAsync();
  }

  switchMetric(
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
    this.currentMin = min;
    this.currentMax = max;
    if (lod !== this.lod) {
      this.lod = lod;
      this.fetchSelectedMetricsAsync();
    }
  }

  private mapToPoints(processMetricReply: ProcessMetricReply): Point[] {
    return processMetricReply.points.map((p) => {
      return <Point>{
        time: p.timeMs,
        value: p.value,
      };
    });
  }

  async fetchSelectedMetricsAsync() {
    let result = await Promise.all(
      this.metrics.map(async (m) => {
        let result = await client.fetch_process_metric({
          lod: this.lod,
          processId: this.processId,
          metricName: m.name,
          beginMs: this.totalMin,
          endMs: this.totalMax,
        });
        return {
          result: result,
          name: m.name,
        };
      })
    );

    this.metricStore.update((metrics) => {
      result.forEach((reply) => {
        let existingMetric = metrics.filter(
          (m) => m.metricDesc.name === reply.name
        )[0];
        if (existingMetric) {
          let index = metrics.indexOf(existingMetric);
          let metric = metrics[index];
          metric.points = this.mapToPoints(reply.result);
          metrics[index] = metric;
        }
      });
      return metrics;
    });
  }
}
