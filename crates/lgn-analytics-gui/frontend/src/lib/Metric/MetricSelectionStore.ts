import { MetricSelectionState } from "@/components/Metric/MetricSelectionState";
import { writable } from "svelte/store";
import { MetricState } from "./MetricState";

const localStorageKey = "last-metric-used";
const recentCount = 3;

export const selectionStore = writable<MetricSelectionState[]>(
  getRecentlyUsedMetrics()
);

export const recentlyUsed = writable(getRecentlyUsedMetrics());

export function getRecentlyUsedMetrics(): MetricSelectionState[] {
  const jsonData = localStorage.getItem(localStorageKey);
  const result = jsonData ? JSON.parse(jsonData) : [];
  return result;
}

function updateRecentlyUsed(state: MetricSelectionState) {
  let used = getRecentlyUsedMetrics();
  if (!used.some((s) => s.name === state.name)) {
    used = [...used, state].slice(-recentCount);
  } else {
    const metric = used.filter((m) => m.name === state.name)[0];
    if (metric) {
      const index = used.indexOf(metric);
      if (index) {
        used[index] = state;
      }
    }
  }
  recentlyUsed.set(used);
  localStorage.setItem(localStorageKey, JSON.stringify(used));
}

export function updateMetricSelection(state: MetricSelectionState) {
  updateRecentlyUsed(state);
  selectionStore.update((data) => {
    const index = data.indexOf(state);
    data[index] = state;
    return data;
  });
}

export function addToSelectionStore(state: MetricState) {
  selectionStore.update((data) => {
    const metric = data.filter((d) => d.name === state.name)[0];
    if (!metric) {
      data = [
        ...data,
        new MetricSelectionState(state.name, state.unit, false, false),
      ];
    }
    return data;
  });
}
