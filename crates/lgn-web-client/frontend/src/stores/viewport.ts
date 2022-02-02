import { Orchestrator, Writable } from "../lib/store";
import MapStore from "./map";

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
  lang: string;
};

export type Video = CommonConfig & {
  type: "video";
};

export type Viewport = Script | Video;

export type ViewportStore = MapStore<Viewport>;

export type AddViewportConfig = {
  /**
   * Focus the newly added viewport.
   */
  focus?: boolean;
};

export default class implements Orchestrator {
  name = "viewport";

  viewportStore = new MapStore<Viewport>();

  activeViewportStore = new Writable<Viewport | null>(null);

  add(
    key: symbol,
    viewport: Viewport,
    { focus }: AddViewportConfig = { focus: false }
  ) {
    this.viewportStore.add(key, viewport);

    if (focus) {
      this.activate(key);
    }
  }

  addAllViewport(...viewportList: [key: symbol, value: Viewport][]) {
    this.viewportStore.addAll(...viewportList);
  }

  activate(key: symbol) {
    this.activeViewportStore.update((activeViewport) => {
      const viewport = this.viewportStore.value.get(key);

      if (!viewport) {
        return activeViewport;
      }

      return viewport;
    });
  }

  remove(key: symbol) {
    const removed = this.viewportStore.remove(key);

    if (removed) {
      this.activeViewportStore.set(
        this.viewportStore.value.entries().next().value?.[1] || null
      );
    }

    return removed;
  }

  removeByValue(viewport: Viewport) {
    let foundKey: symbol | null = null;

    for (const [key, value] of this.viewportStore.value) {
      if (value === viewport) {
        foundKey = key;
      }
    }

    if (!foundKey) {
      return false;
    }

    return this.remove(foundKey);
  }
}
