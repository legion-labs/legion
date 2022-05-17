import type { LayoutConfig } from "golden-layout";
import PropertyGrid from "../propertyGrid/PropertyGrid.svelte";
import OrangeSvelte from "./Orange.svelte";
import type { SvelteComponentDev } from "svelte/internal";
import ResourceBrowser from "../ResourceBrowser.svelte";
import SceneExplorer from "../SceneExplorer.svelte";

export const AppComponentMap: Record<string, typeof SvelteComponentDev> = {
  PropertyGrid: PropertyGrid,
  ResourceBrowser: ResourceBrowser,
  SceneExplorer: SceneExplorer,
  Orange: OrangeSvelte,
};

export const layoutConfig: LayoutConfig = {
  settings: {
    showPopoutIcon: false,
  },
  dimensions: {
    minItemHeight: 100,
    minItemWidth: 100,
  },
  root: {
    type: "column",
    content: [
      {
        type: "component",
        componentType: "PropertyGrid",
        title: "Property Grid",
        minWidth: 100,
        minHeight: 100,
      },
      {
        type: "component",
        componentType: "PropertyGrid",
        title: "Property Grid",
        minWidth: 100,
        minHeight: 100,
      },
      {
        type: "component",
        componentType: "Orange",
        minWidth: 10,
        minHeight: 100,
      },
      {
        type: "component",
        componentType: "Orange",
        minHeight: 100,
      },
    ],
  },
};
