import { derived, get, writable } from "svelte/store";
import type { Writable } from "svelte/store";

import type { MetricBlockData } from "@lgn/proto-telemetry/dist/metric";
import { DefaultLocalStorage } from "@lgn/web-client/src/lib/storage";
import { connected } from "@lgn/web-client/src/lib/store";

import type { MetricConfig } from "./MetricConfig";
import type { MetricState } from "./MetricState";

/** All the recently used metrics  */
const recentlyUsedMetricsStoreKey = "recently-used-metric-store-key";

/** Last used metrics for each process ids */
const lastUsedMetricStoreKey = "last-used-metric-store-key";

export type MetricStore = ReturnType<typeof getMetricStore>;

export type RecentlyUsedMetricStore = ReturnType<
  typeof getRecentlyUsedMetricsStore
>;

export type LastUsedMetricsStore = ReturnType<typeof getLastUsedMetricsStore>;

export type MetricConfigStore = ReturnType<typeof getMetricConfigStore>;

export function getMetricConfigStore(): Writable<MetricConfig[]> {
  const store = connected<typeof recentlyUsedMetricsStoreKey, MetricConfig[]>(
    new DefaultLocalStorage(),
    recentlyUsedMetricsStoreKey,
    []
  );

  const { subscribe } = derived(store, ($recentlyUsedMetrics) =>
    [...$recentlyUsedMetrics].sort((a, b) => a.lastUse - b.lastUse)
  );

  return {
    update: store.update,
    set: store.set,
    subscribe,
  };
}

export function getLastUsedMetricsStore() {
  const store = connected<typeof lastUsedMetricStoreKey, string[]>(
    new DefaultLocalStorage(),
    lastUsedMetricStoreKey,
    []
  );

  return {
    ...store,

    clearMetrics(clearedMetricNames: string[]) {
      store.update((metricNames) =>
        metricNames.filter(
          (metricName) => !clearedMetricNames.includes(metricName)
        )
      );
    },

    toggleMetric(toggledMetricName: string) {
      store.update((metricNames) => {
        if (metricNames.includes(toggledMetricName)) {
          return metricNames.filter(
            (metricName) => metricName !== toggledMetricName
          );
        }

        return [...metricNames, toggledMetricName];
      });
    },
  };
}

export function getRecentlyUsedMetricsStore(
  metricStore: MetricStore,
  metricConfigStore: MetricConfigStore
) {
  return derived(
    [metricStore, metricConfigStore],
    ([$metrics, $metricConfig]) => {
      const result: MetricState[] = [];

      [...$metrics]
        .sort((a, b) => (b.lastUse ?? 0) - (a.lastUse ?? 0))
        .forEach((m) => {
          if ($metricConfig.some((l) => l.name === m.name)) {
            result.push(m);
          }
        });

      return result.slice(0, 5);
    }
  );
}

export function getMetricStore(
  lastUsedMetricsStore: LastUsedMetricsStore,
  metricConfigStore: MetricConfigStore
) {
  const metricsStore = writable<MetricState[]>([]);

  const { subscribe } = derived(metricsStore, ($metrics) =>
    $metrics.sort((metric1, metric2) => (metric1.name > metric2.name ? 1 : -1))
  );

  const registerMetrics = (metrics: MetricState[]) => {
    // Todo : apply the selected status using the local storage config
    metricsStore.set(metrics);
  };

  const updateSerialize = (action: (state: MetricState[]) => void) => {
    metricsStore.update((s) => {
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
      const merge = get(metricConfigStore).concat(config);
      metricConfigStore.set(merge);
      return s;
    });
  };

  const registerBlock = (
    blockData: MetricBlockData,
    blockId: string,
    metricName: string
  ) => {
    metricsStore.update((metrics) => {
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
    lastUsedMetricsStore.toggleMetric(name);

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
    metricsStore.update((s) => {
      lastUsedMetricsStore.clearMetrics(s.map((m) => m.name));

      s.forEach((e) => {
        e.selected = false;
      });
      return s;
    });
  };

  const switchHidden = (name: string) => {
    metricsStore.update((s) => {
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
