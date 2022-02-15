import { Point } from "@/lib/Metric/MetricPoint";
import {
  MetricBlockData,
  MetricManifest,
  ProcessMetricReply,
} from "@lgn/proto-telemetry/dist/metric";
import { MetricBlockState } from "./MetricBlockState";

export class MetricState {
  enabled: boolean;
  name: string;
  unit: string;
  min: number;
  max: number;
  private manifest: MetricManifest;
  private blocks: Map<string, MetricBlockState>;
  constructor(enabled: boolean, manifest: MetricManifest) {
    this.enabled = enabled;
    this.manifest = manifest;
    this.name = this.manifest.name;
    this.unit = this.manifest.unit;
    this.blocks = new Map();
    for (const block of this.manifest.blocks) {
      this.blocks.set(block.blockId, new MetricBlockState(block));
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

  *getMissingBlocks(minMs: number, maxMs: number, lod: number) {
    for (const block of [...this.getViewportBlocks(minMs, maxMs)]) {
      if (!block.hasLod(lod)) {
        yield block;
      }
    }
  }

  *getViewportPoints(min: number, max: number, lod: number) {
    for (const block of this.getViewportBlocks(min, max)) {
      let blockState = this.blocks.get(block.blockId);
      if (blockState) {
        for (const point of blockState.getPoints(min, max, lod)) {
          yield point;
        }
      }
    }
  }

  store(processMetricReply: ProcessMetricReply): boolean {
    let mutated = false;
    for (const blockData of processMetricReply.blocks) {
      const block = this.blocks.get(blockData.blockId);
      if (block) {
        if (block.store(blockData.lod, this.mapToPoints(blockData))) {
          mutated = true;
        }
      }
    }
    return mutated;
  }

  private mapToPoints(blockData: MetricBlockData): Point[] {
    return blockData.points.map((p) => {
      return <Point>{
        time: p.timeMs + this.min,
        value: p.value,
      };
    });
  }
}
