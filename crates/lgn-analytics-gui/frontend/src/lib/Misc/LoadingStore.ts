import { Subscriber, Writable, writable } from "svelte/store";

export const loadingStore = createLoadStore();

type LoadingState = {
  requested: number;
  completed: number;
};

export type LoadingStore = {
  subscribe: Writable<LoadingState>["subscribe"];
  reset(): void;
  addWork(): void;
  completeWork(): void;
};

function createLoadStore(): LoadingStore {
  const { subscribe, set, update } = writable<LoadingState>({
    completed: 0,
    requested: 0,
  });

  return {
    subscribe,
    reset: () =>
      set({
        completed: 0,
        requested: 0,
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

export async function loadWrap<T>(action: () => T): Promise<T> {
  const store = loadingStore;

  try {
    store.addWork();
    return await action();
  } finally {
    store.completeWork();
  }
}
