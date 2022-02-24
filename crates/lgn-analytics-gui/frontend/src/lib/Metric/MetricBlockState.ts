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
    const data = this.data.get(Math.max(lod, Math.min(...this.data.keys())));
    if (!data) {
      return;
    }

    const points = [];
    for (const point of data) {
      if (point.time >= min && point.time <= max) {
        points.push(point);
      }
    }

    if (points.length > 0) {
      if (data.length != points.length) {
        const boundaryInPoint = data[data.indexOf(points[0]) - 1];
        if (boundaryInPoint && !points.includes(boundaryInPoint)) {
          points.unshift(boundaryInPoint);
        }
        const boundaryOutPoint =
          data[data.indexOf(points[points.length - 1]) + 1];
        if (boundaryOutPoint && !points.includes(boundaryOutPoint)) {
          points.push(boundaryOutPoint);
        }
      }
    } else {
      const nextMinPoint = data.filter((p) => p.time <= min)[0];
      if (nextMinPoint && !points.includes(nextMinPoint)) {
        points.push(nextMinPoint);
      }
      const nextMaxPoint = data.filter((p) => p.time >= max)[0];
      if (nextMaxPoint && !points.includes(nextMaxPoint)) {
        points.push(nextMaxPoint);
      }
    }

    for (const p of points) {
      yield p;
    }
  }
}
