import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

export const loadingStore = createLoadStore();

type LoadingState = {
  requested: number;
  completed: number;
  scale: number;
};

export type LoadingStore = {
  subscribe: Writable<LoadingState>["subscribe"];
  reset(scale: number): void;
  addWork(): void;
  completeWork(): void;
};

function createLoadStore(): LoadingStore {
  const { subscribe, set, update } = writable<LoadingState>({
    completed: 0,
    requested: 0,
    scale: 1,
  });

  return {
    subscribe,
    reset: (scale) =>
      set({
        completed: 0,
        requested: 0,
        scale: Math.min(1, scale),
      }),
    addWork: () =>
      update((s) => {
        s.requested++;
        return s;
      }),
    completeWork: () =>
      update((s) => {
        s.completed++;
        return s;
      }),
  };
}

export function loadWrap<T>(action: () => Promise<T>): Promise<T> {
  const store = loadingStore;

  try {
    store.addWork();
    return action();
  } finally {
    store.completeWork();
  }
}

export function loadPromise<T>(p: Promise<T>): Promise<T> {
  const store = loadingStore;
  store.addWork();

  return (async () => {
    const res = await p;
    store.completeWork();
    return res;
  })();
}
