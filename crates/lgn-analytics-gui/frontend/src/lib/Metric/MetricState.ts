import { Point } from "@/lib/Metric/MetricPoint";
import {
  MetricBlockData,
  MetricBlockManifest,
  MetricDesc,
} from "@lgn/proto-telemetry/dist/metric";
import { MetricBlockState } from "./MetricBlockState";

export class MetricState {
  enabled: boolean;
  name: string;
  unit: string;
  minTick = NaN;
  maxTick = NaN;
  private blocks: Map<string, MetricBlockState>;
  private startTick: number;
  constructor(enabled: boolean, metricDesc: MetricDesc, startTick: number) {
    this.enabled = enabled;
    this.name = metricDesc.name;
    this.unit = metricDesc.unit;
    this.blocks = new Map();
    this.startTick = startTick;
  }

  registerBlock(manifest: MetricBlockManifest) {
    if (manifest.desc) {
      this.blocks.set(
        manifest.desc.blockId,
        new MetricBlockState(manifest.desc)
      );
    }
    this.minTick = Math.min(
      ...Array.from(this.blocks.values()).map((v) => v.minTick)
    );
    this.maxTick = Math.max(
      ...Array.from(this.blocks.values()).map((b) => b.maxTick)
    );
  }

  *getViewportBlocks(min: number, max: number) {
    for (const [_, value] of this.blocks) {
      if (value.isInViewport(min, max)) {
        yield value;
      }
    }
  }

  *requestMissingBlocks(min: number, max: number, lod: number) {
    for (const block of [...this.getViewportBlocks(min, max)]) {
      if (block.requestLod(lod)) {
        yield block;
      }
    }
  }

  *getViewportPoints(min: number, max: number, lod: number) {
    for (const block of this.getViewportBlocks(min, max)) {
      const blockState = this.blocks.get(block.blockId);
      if (blockState) {
        for (const point of blockState.getPoints(min, max, lod)) {
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

  private mapToPoints(blockData: MetricBlockData): Point[] {
    return blockData.points.map((p) => {
      return new Point(p.tickOffset - this.startTick, p.value);
    });
  }
}
