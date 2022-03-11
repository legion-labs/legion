import type { Writable } from "svelte/store";
import { get, writable } from "svelte/store";
import { createMapStore } from "../stores/map";

type CommonConfig = {
  name: string;
  /**
   * Whether or not the added viewport can be "removed" from the store.
   * This config might get removed later on as all viewport will be "removable".
   */
  removable?: boolean;
};

export type Script = CommonConfig & {
  type: "script";
  onChange(newValue: string): void;
  value: string;
  readonly?: boolean;
  lang: string;
};

export type Video = CommonConfig & {
  type: "video";
};

export type Viewport = Script | Video;

export type ViewportStore = Writable<Map<symbol, Viewport>>;

export type AddViewportConfig = {
  /**
   * Focus the newly added viewport.
   */
  focus?: boolean;
};

export function createViewportOrchestrator() {
  return {
    viewportStore: createMapStore<Viewport>(),

    activeViewportStore: writable<Viewport | null>(null),

    add(
      key: symbol,
      viewport: Viewport,
      { focus }: AddViewportConfig = { focus: false }
    ) {
      this.viewportStore.add(key, viewport);

      if (focus) {
        this.activate(key);
      }
    },

    addAllViewport(...viewportList: [key: symbol, value: Viewport][]) {
      this.viewportStore.addAll(...viewportList);
    },

    activate(key: symbol) {
      this.activeViewportStore.update((activeViewport) => {
        const viewport = get(this.viewportStore).get(key);

        if (!viewport) {
          return activeViewport;
        }

        return viewport;
      });
    },

    remove(key: symbol) {
      const removed = this.viewportStore.remove(key);

      if (removed) {
        this.activeViewportStore.set(
          get(this.viewportStore).entries().next().value?.[1] || null
        );
      }

      return removed;
    },

    removeByValue(viewport: Viewport) {
      let foundKey: symbol | null = null;

      for (const [key, value] of get(this.viewportStore)) {
        if (value === viewport) {
          foundKey = key;
        }
      }

      if (!foundKey) {
        return false;
      }

      return this.remove(foundKey);
    },
  };
}
