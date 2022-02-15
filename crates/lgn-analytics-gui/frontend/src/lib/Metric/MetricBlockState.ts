import { Point } from "@/lib/Metric/MetricPoint";
import { MetricBlockDesc } from "@lgn/proto-telemetry/dist/metric";

export class MetricBlockState {
  blockId: string;
  streamId: string;
  minMs: number;
  maxMs: number;
  private data: Map<number, Point[]>;
  private inFlight: Map<number, boolean>;
  constructor(metricBlockDesc: MetricBlockDesc) {
    this.blockId = metricBlockDesc.blockId;
    this.streamId = metricBlockDesc.streamId;
    this.minMs = metricBlockDesc.beginTimeMs;
    this.maxMs = metricBlockDesc.endTimeMs;
    this.data = new Map();
    this.inFlight = new Map();
  }

  hasLod(lod: number) {
    return this.data.get(lod) !== undefined;
  }

  requestLod(lod: number) {
    if (this.hasLod(lod)) {
      return false;
    }
    if (this.inFlight.get(lod)) {
      return false;
    }
    this.inFlight.set(lod, true);
    return true;
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
