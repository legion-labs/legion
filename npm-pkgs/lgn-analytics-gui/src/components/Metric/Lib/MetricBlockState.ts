import type { MetricBlockDesc } from "@lgn/proto-telemetry/dist/metric";

import type { MetricPoint } from "./MetricPoint";

export class MetricBlockState {
  blockId: string;
  streamId: string;
  minMs: number;
  maxMs: number;
  private data: Map<number, MetricPoint[]>;
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
    if (this.inFlight.get(lod) !== undefined) {
      return false;
    }
    this.inFlight.set(lod, true);
    return true;
  }

  store(lod: number, points: MetricPoint[]): boolean {
    if (this.hasLod(lod)) {
      return false;
    }
    this.data.set(lod, points);
    return true;
  }

  isInViewport(min: number, max: number) {
    return !(this.minMs > max || this.maxMs < min);
  }

  *getPoints(min: number, max: number, lod: number, withBoundaries: boolean) {
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

    if (withBoundaries) {
      if (points.length > 0) {
        const boundaryInPoint = data[data.indexOf(points[0]) - 1];
        if (
          boundaryInPoint !== undefined &&
          !points.includes(boundaryInPoint)
        ) {
          points.unshift(boundaryInPoint);
        }
        const boundaryOutPoint =
          data[data.indexOf(points[points.length - 1]) + 1];
        if (
          boundaryOutPoint !== undefined &&
          !points.includes(boundaryOutPoint)
        ) {
          points.push(boundaryOutPoint);
        }
      } else {
        const nextMinPoint = data
          .filter((p) => p.time <= min)
          .sort((a, b) => (a > b ? -1 : 1))[0];
        if (nextMinPoint !== undefined && !points.includes(nextMinPoint)) {
          points.push(nextMinPoint);
        }
        const nextMaxPoint = data
          .filter((p) => p.time >= max)
          .sort((a, b) => (a > b ? 1 : -1))[0];
        if (nextMaxPoint !== undefined && !points.includes(nextMaxPoint)) {
          points.push(nextMaxPoint);
        }
      }
    }

    for (const p of points) {
      yield p;
    }
  }
}
