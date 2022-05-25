import { derived, writable } from "svelte/store";

import type { MetricBlockData } from "@lgn/proto-telemetry/dist/metric";
import { DefaultLocalStorage } from "@lgn/web-client/src/lib/storage";
import { connected } from "@lgn/web-client/src/lib/store";

import type { MetricConfig } from "./MetricConfig";
import type { MetricState } from "./MetricState";

/** All the recently used metrics  */
const localStorageKey = "metric-config";

/** Last used metrics for each process ids */
const recentlyUsedMetricStoreKey = "last-used-metric-store-key";

export type MetricStore = ReturnType<typeof getMetricStore>;

export type RecentlyUsedMetricStore = ReturnType<typeof getRecentlyUsedStore>;

export type LastMetricsUsedStore = ReturnType<typeof getLastMetricsUsedStore>;

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

export function getLastMetricsUsedStore() {
  const store = connected<
    typeof recentlyUsedMetricStoreKey,
    Record<string, string[] | undefined>
  >(new DefaultLocalStorage(), recentlyUsedMetricStoreKey, {});

  return {
    ...store,
    initProcess(processId: string) {
      store.update((processes) => {
        const metrics = processes[processId];

        return {
          ...processes,
          [processId]: Array.isArray(metrics) ? metrics : [],
        };
      });
    },

    toggleMetricForProcess(processId: string, metricName: string) {
      store.update((processes) => {
        const metrics = processes[processId];

        if (!metrics) {
          return {
            ...processes,
            [processId]: [metricName],
          };
        }

        if (metrics.includes(metricName)) {
          return {
            ...processes,
            [processId]: metrics.filter((metric) => metric !== metricName),
          };
        }

        return {
          ...processes,
          [processId]: [...metrics, metricName],
        };
      });
    },
  };
}

export function getMetricStore(
  processId: string,
  lastMetricsUsedStore: LastMetricsUsedStore
) {
  const { subscribe, set, update } = writable<MetricState[]>([]);

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
    lastMetricsUsedStore.toggleMetricForProcess(processId, name);

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
