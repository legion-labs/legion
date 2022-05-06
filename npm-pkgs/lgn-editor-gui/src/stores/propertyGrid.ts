import { createMapStore } from "@lgn/web-client/src/stores/map";

export type PropertyGridStore = ReturnType<typeof createPropertyGridStore>;

export function createPropertyGridStore() {
  const { subscribe, update } = createMapStore<symbol, boolean>();

  const switchCollapse = (key: symbol) => {
    update((s) => {
      s.set(key, s.has(key) ? !s.get(key) : true);

      return s;
    });
  };

  return { subscribe, switchCollapse };
}
