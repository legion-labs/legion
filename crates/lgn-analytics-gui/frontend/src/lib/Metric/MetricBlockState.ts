import { Point } from "@/lib/Metric/MetricPoint";
import { MetricBlockDesc } from "@lgn/proto-telemetry/dist/metric";
import { Numeric } from "d3";

export class MetricBlockState {
  blockId: string;
  streamId: string;
  minMs: number;
  maxMs: number;
  private data: Map<Numeric, Point[]>;
  constructor(metricBlockDesc: MetricBlockDesc) {
    this.blockId = metricBlockDesc.blockId;
    this.streamId = metricBlockDesc.streamId;
    this.minMs = metricBlockDesc.beginTimeMs;
    this.maxMs = metricBlockDesc.endTimeMs;
    this.data = new Map();
  }

  hasLod(lod: number) {
    return this.data.get(lod) !== undefined;
  }

  store(lod: number, points: Point[]): boolean {
    if (this.hasLod(lod)) {
      return false;
    }
    this.data.set(lod, points);
    return true;
  }

  isInViewport(min: number, max: number) {
    return !(this.minMs > max || this.maxMs < min);
  }

  *getPoints(min: number, max: number, lod: number) {
    const data = this.data.get(lod);
    if (data) {
      for (let index = 0; index < data.length; index++) {
        const point = data[index];
        if (point.time >= min && point.time <= max) {
          yield data[index];
        }
      }
    }
  }
}
