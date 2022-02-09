import { client } from "@/lib/client";
import { Point } from "@/lib/point";
import {
  MetricDesc,
  ProcessMetricReply,
} from "@lgn/proto-telemetry/dist/metric";
import { Writable, writable } from "svelte/store";

export class MetricStreamer {
  private lod: number;
  private totalMin: number;
  private totalMax: number;
  private processId: string;
  private currentMin: number = -Infinity;
  private currentMax: number = Infinity;
  private metrics: MetricDesc[];
  metricStore: Writable<MetricDesc[]>;
  points: Writable<Point[][]>;
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
    this.points = writable([]);
  }

  async initializeAsync() {
    this.metrics = (
      await client.list_process_metrics({
        processId: this.processId,
      })
    ).metrics;

    this.metricStore.set(this.metrics);

    await this.fetchSelectedMetricsAsync();
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
      this.metrics.map((m) => {
        return client.fetch_process_metric({
          lod: this.lod,
          processId: this.processId,
          metricName: m.name,
          beginMs: this.totalMin,
          endMs: this.totalMax,
        });
      })
    );

    this.points.set(result.map((m) => this.mapToPoints(m)));
  }
}
