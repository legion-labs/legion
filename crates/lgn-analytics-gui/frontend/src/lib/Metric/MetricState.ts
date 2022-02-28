import { Point } from "@/lib/Metric/MetricPoint";
import {
  MetricBlockData,
  MetricBlockManifest,
  MetricDesc,
} from "@lgn/proto-telemetry/dist/metric";
import { get } from "svelte/store";
import { MetricBlockState } from "./MetricBlockState";
import { addToSelectionStore, selectionStore } from "./MetricSelectionStore";

export class MetricState {
  name: string;
  unit: string;
  min = -Infinity;
  max = Infinity;
  private blocks: Map<string, MetricBlockState>;
  constructor(metricDesc: MetricDesc) {
    this.name = metricDesc.name;
    this.unit = metricDesc.unit;
    this.blocks = new Map();
    addToSelectionStore(this);
  }

  canBeDisplayed(): boolean {
    const metric = get(selectionStore).filter((m) => m.name === this.name)[0];
    if (!metric) {
      return false;
    }
    return metric.selected && !metric.hidden;
  }

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
      return <Point>{
        time: p.timeMs + this.min,
        value: p.value,
      };
    });
  }

  getClosestValue(time: number): Point | null {
    const threshold = 100;
    const data = Array.from(
      this.getViewportPoints(time - threshold, time + threshold, 0)
    );
    if (data.length <= 0) {
      return null;
    }
    const result = data.reduce((p, c) =>
      Math.abs(c.time - time) < Math.abs(p.time - time) ? c : p
    );
    return result;
  }
}
