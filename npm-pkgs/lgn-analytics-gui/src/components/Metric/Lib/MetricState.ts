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
  selected;
  lastUse: number | null = null;

  #blocks: Map<string, MetricBlockState> = new Map();

  constructor(
    metricDesc: MetricDesc,
    { selected = true }: { selected?: boolean } = {}
  ) {
    this.name = metricDesc.name;
    this.unit = metricDesc.unit;
    this.selected = selected;
  }

  canBeDisplayed = () => this.selected && !this.hidden;

  registerBlock(manifest: MetricBlockManifest) {
    if (manifest.desc) {
      this.#blocks.set(
        manifest.desc.blockId,
        new MetricBlockState(manifest.desc)
      );
    }
    this.min = Math.min(
      ...Array.from(this.#blocks.values()).map((v) => v.minMs)
    );
    this.max = Math.max(
      ...Array.from(this.#blocks.values()).map((b) => b.maxMs)
    );
  }

  *getViewportBlocks(minMs: number, maxMs: number) {
    for (const [_, value] of this.#blocks) {
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
      const blockState = this.#blocks.get(block.blockId);
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
    const block = this.#blocks.get(blockId);
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

  getClosestValue({
    time,
    min,
    max,
    lod,
  }: {
    time: number;
    min: number;
    max: number;
    lod: number;
  }): MetricPoint | null {
    let result: MetricPoint | null = null;

    for (const point of this.getViewportPoints(min, max, lod, false)) {
      if (!result) {
        result = point;
        continue;
      }

      if (Math.abs(time - point.time) < Math.abs(time - result.time)) {
        result = point;
      }
    }

    return result;
  }
}
