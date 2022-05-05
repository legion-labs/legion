import { writable } from "svelte/store";

export type PropertyGridStore = ReturnType<typeof createPropertyGridStore>;

export function createPropertyGridStore() {
  const { subscribe, update } = writable<Record<symbol, boolean>>({});

  const switchCollapse = (key: symbol) => {
    update((s) => {
      s[key] = !s[key];

      return s;
    });
  };

  return { subscribe, switchCollapse };
}
