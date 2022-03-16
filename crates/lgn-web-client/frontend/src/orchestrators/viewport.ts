import type { Writable } from "svelte/store";
import { get, writable } from "svelte/store";
import type { MapStore } from "../stores/map";
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

// export type ViewportStore = Writable<Map<symbol, Viewport>>;

export type AddViewportConfig = {
  /**
   * Focus the newly added viewport.
   */
  focus?: boolean;
};

export type ViewportOrchestrator = {
  viewportStore: MapStore<symbol, Viewport>;
  activeViewportStore: Writable<Viewport | null>;
  add(key: symbol, viewport: Viewport, { focus }: AddViewportConfig): void;
  addAllViewport(...viewportList: [key: symbol, value: Viewport][]): void;
  activate(key: symbol): void;
  remove(key: symbol): boolean;
  removeByValue(viewport: Viewport): boolean;
};

export function createViewportOrchestrator(): ViewportOrchestrator {
  return {
    viewportStore: createMapStore(),

    activeViewportStore: writable(null),

    add(key, viewport, { focus } = { focus: false }) {
      this.viewportStore.add(key, viewport);

      if (focus) {
        this.activate(key);
      }
    },

    addAllViewport(...viewportList) {
      this.viewportStore.addAll(...viewportList);
    },

    activate(key) {
      this.activeViewportStore.update((activeViewport) => {
        const viewport = get(this.viewportStore).get(key);

        if (!viewport) {
          return activeViewport;
        }

        return viewport;
      });
    },

    remove(key) {
      const removed = this.viewportStore.remove(key);

      if (removed) {
        this.activeViewportStore.set(
          get(this.viewportStore).entries().next().value?.[1] || null
        );
      }

      return removed;
    },

    removeByValue(viewport) {
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
