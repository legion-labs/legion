import * as d3 from "d3";

export function getMetricColor(name: string) {
  const color = Math.abs(hashString(name)) % 10;
  return d3.schemeCategory10[color];
}

function hashString(string: string): number {
  let hash = 0;
  for (let i = 0; i < string.length; i++) {
    hash = string.charCodeAt(i) + ((hash << 5) - hash);
    hash = hash & hash;
  }
  return hash;
}
