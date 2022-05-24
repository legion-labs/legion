import { derived, writable } from "svelte/store";

import type { MetricBlockData } from "@lgn/proto-telemetry/dist/metric";

import type { MetricConfig } from "./MetricConfig";
import type { MetricState } from "./MetricState";

const localStorageKey = "metric-config";

export type MetricStore = ReturnType<typeof getMetricStore>;

function getMetricConfig(): MetricConfig[] {
  const jsonData = localStorage.getItem(localStorageKey);
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const data: MetricConfig[] = jsonData !== null ? JSON.parse(jsonData) : [];
  return data.sort((a, b) => a.lastUse - b.lastUse);
}

export function getRecentlyUsedStore(metricStore: MetricStore) {
  return derived(metricStore, (data) => {
    const local = getMetricConfig();
    const result: MetricState[] = [];
    data.forEach((m) => {
      if (local.some((l) => l.name === m.name)) {
        result.push(m);
      }
    });
    return result.slice(0, 5);
  });
}

export function getMetricStore() {
  const { subscribe, set, update } = writable<MetricState[]>();

  const registerMetrics = (metrics: MetricState[]) => {
    // Todo : apply the selected status using the local storage config
    set(metrics);
  };

  const updateSerialize = (action: (state: MetricState[]) => void) => {
    update((s) => {
      action(s);
      const data = s.filter((s) => s.selected);
      const config: MetricConfig[] = [];
      data.forEach((m) => {
        if (m.lastUse !== null) {
          config.push({
            name: m.name,
            lastUse: m.lastUse,
          });
        }
      });
      const merge = getMetricConfig().concat(config);
      localStorage.setItem(localStorageKey, JSON.stringify(merge));
      return s;
    });
  };

  const registerBlock = (
    blockData: MetricBlockData,
    blockId: string,
    metricName: string
  ) => {
    update((metrics) => {
      const m = metrics.find((m) => m.name === metricName);
      if (m) {
        const index = metrics.indexOf(m);
        if (m.store(blockId, blockData)) {
          metrics[index] = m;
        }
      }
      return metrics;
    });
  };

  const switchSelection = (name: string) => {
    updateSerialize((s) => {
      const metric = s.find((d) => d.name === name);
      if (metric) {
        metric.selected = !metric.selected;
        if (metric.selected) {
          metric.lastUse = Date.now();
          metric.hidden = false;
        }
      }
      return s;
    });
  };

  const clearSelection = () => {
    update((s) => {
      s.forEach((e) => {
        e.selected = false;
      });
      return s;
    });
  };

  const switchHidden = (name: string) => {
    update((s) => {
      const metric = s.find((d) => d.name === name);
      if (metric) {
        metric.hidden = !metric.hidden;
      }
      return s;
    });
  };

  return {
    subscribe,
    registerMetrics,
    registerBlock,
    switchSelection,
    switchHidden,
    clearSelection,
  };
}
