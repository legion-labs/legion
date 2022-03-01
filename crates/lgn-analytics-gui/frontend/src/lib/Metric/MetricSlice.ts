import { Point } from "./MetricPoint";

export interface MetricSlice {
  unit: string;
  name: string;
  points: Point[];
}
