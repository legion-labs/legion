import * as d3 from "d3";

import { MetricAxis } from "./MetricAxis";
import type { MetricSlice } from "./MetricSlice";
import type { MetricState } from "./MetricState";

export class MetricAxisCollection {
  private data: Map<string, MetricAxis>;
  constructor() {
    this.data = new Map();
  }

  getBestAxisScale(
    range: [number, number],
    metrics: MetricState[]
  ): d3.ScaleLinear<number, number, never> {
    const bestAxis = this.getBestAxis(metrics);

    return bestAxis
      ? bestAxis.scale.range(range)
      : d3.scaleLinear().range(range).nice();
  }

  getAxisScale(
    unit: string,
    range: [number, number]
  ): d3.ScaleLinear<number, number, never> {
    const result = this.data.get(unit);
    if (result) {
      return result.scale.range(range);
    }
    return d3.scaleLinear().range(range);
  }

  update(slices: MetricSlice[]) {
    for (const slice of slices) {
      let axis = this.data.get(slice.unit);
      if (!axis) {
        axis = new MetricAxis(slice.unit);
        this.data.set(slice.unit, axis);
      }
      axis.update(slices);
    }
  }

  private getBestAxis(metrics: MetricState[]): MetricAxis | undefined {
    const result = metrics.map((metric) => ({
      unit: metric.unit,
      count: metrics.filter(
        (m) => m.selected && !m.hidden && m.unit === metric.unit
      ).length,
    }));
    const axis = result.sort((a, b) => (b.count > a.count ? 1 : -1))[0];
    return this.data.get(axis.unit);
  }
}
