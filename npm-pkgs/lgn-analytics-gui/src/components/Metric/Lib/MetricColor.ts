import { schemeCategory10 } from "d3";

export function getMetricColor(index: number) {
  return schemeCategory10[index % schemeCategory10.length];
}
