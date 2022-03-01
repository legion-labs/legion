import { MetricSlice } from "./MetricSlice";
import * as d3 from "d3";

export class MetricAxis {
  scale: d3.ScaleLinear<number, number, never>;
  private unit: string;
  private min = Infinity;
  private max = -Infinity;
  constructor(unit: string) {
    this.unit = unit;
    this.scale = d3.scaleLinear();
  }
  update(slices: MetricSlice[]) {
    this.min = Infinity;
    this.max = -Infinity;
    for (const slice of slices) {
      if (slice.unit === this.unit) {
        for (const point of slice.points) {
          this.min = Math.min(this.min, point.value);
          this.max = Math.max(this.max, point.value);
        }
      }
    }
    this.scale = this.scale.domain([this.min, this.max]);
  }
}
