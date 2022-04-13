import type { MetricPoint } from "./MetricPoint";

export interface MetricSlice {
  unit: string;
  name: string;
  points: MetricPoint[];
}
