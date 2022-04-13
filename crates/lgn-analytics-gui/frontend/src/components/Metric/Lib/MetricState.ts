import * as d3 from "d3";

import type {
  MetricBlockData,
  MetricBlockManifest,
  MetricDesc,
} from "@lgn/proto-telemetry/dist/metric";

import { MetricBlockState } from "./MetricBlockState";
import type { MetricPoint } from "./MetricPoint";

export class MetricState {
  name: string;
  unit: string;
  min = -Infinity;
  max = Infinity;
  hidden = false;
  selected = true;
  lastUse: number | null = null;
  private blocks: Map<string, MetricBlockState> = new Map();
  constructor(metricDesc: MetricDesc) {
    this.name = metricDesc.name;
    this.unit = metricDesc.unit;
  }

  canBeDisplayed = () => this.selected && !this.hidden;

  registerBlock(manifest: MetricBlockManifest) {
    if (manifest.desc) {
      this.blocks.set(
        manifest.desc.blockId,
        new MetricBlockState(manifest.desc)
      );
    }
    this.min = Math.min(
      ...Array.from(this.blocks.values()).map((v) => v.minMs)
    );
    this.max = Math.max(
      ...Array.from(this.blocks.values()).map((b) => b.maxMs)
    );
  }

  *getViewportBlocks(minMs: number, maxMs: number) {
    for (const [_, value] of this.blocks) {
      if (value.isInViewport(minMs, maxMs)) {
        yield value;
      }
    }
  }

  *requestMissingBlocks(minMs: number, maxMs: number, lod: number) {
    for (const block of [...this.getViewportBlocks(minMs, maxMs)]) {
      if (block.requestLod(lod)) {
        yield block;
      }
    }
  }

  *getViewportPoints(
    min: number,
    max: number,
    lod: number,
    withBoundaries: boolean
  ) {
    for (const block of this.getViewportBlocks(min, max)) {
      const blockState = this.blocks.get(block.blockId);
      if (blockState) {
        for (const point of blockState.getPoints(
          min,
          max,
          lod,
          withBoundaries
        )) {
          yield point;
        }
      }
    }
  }

  store(blockId: string, metricBlockData: MetricBlockData): boolean {
    let mutated = false;
    const block = this.blocks.get(blockId);
    if (block) {
      if (block.store(metricBlockData.lod, this.mapToPoints(metricBlockData))) {
        mutated = true;
      }
    }
    return mutated;
  }

  private mapToPoints(blockData: MetricBlockData): MetricPoint[] {
    return blockData.points.map((p) => {
      return {
        time: p.timeMs + this.min,
        value: p.value,
      } as MetricPoint;
    });
  }

  getClosestValue(time: number): MetricPoint | null {
    let result = null;
    for (const item of this.getViewportPoints(
      time - this.min / 100,
      time + this.max / 100,
      0,
      false
    )) {
      if (item.time > time) {
        if (!result) {
          result = item;
        }
        if (item.time < result.time) {
          result = item;
        }
      }
    }
    return result;
  }

  static getMetricColor(name: string) {
    const color = Math.abs(this.hashString(name)) % 10;
    return d3.schemeCategory10[color];
  }

  private static hashString(string: string): number {
    let hash = 0;
    for (let i = 0; i < string.length; i++) {
      hash = string.charCodeAt(i) + ((hash << 5) - hash);
      hash = hash & hash;
    }
    return hash;
  }
}
