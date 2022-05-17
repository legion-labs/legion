import type { LayoutConfig } from "golden-layout";
import PropertyGrid from "../propertyGrid/PropertyGrid.svelte";
import OrangeSvelte from "./Orange.svelte";
import type { SvelteComponentDev } from "svelte/internal";
import ResourceBrowser from "../ResourceBrowser.svelte";
import SceneExplorer from "../SceneExplorer.svelte";
import RemoteWindowSvelte from "@lgn/web-client/src/components/RemoteWindow.svelte";
import LocalChanges from "../localChanges/LocalChanges.svelte";
import Log from "../Log.svelte";

export const AppComponentMap: Record<string, typeof SvelteComponentDev> = {
  PropertyGrid: PropertyGrid,
  ResourceBrowser: ResourceBrowser,
  SceneExplorer: SceneExplorer,
  Orange: OrangeSvelte,
  RemoteWindow: RemoteWindowSvelte,
  LocalChanges: LocalChanges,
  Log: Log,
};

export const defaultLayoutConfig: LayoutConfig = {
  settings: {
    showPopoutIcon: false,
  },
  dimensions: {
    minItemHeight: 100,
    minItemWidth: 100,
  },
  root: {
    type: "row",
    content: [
      {
        type: "column",
        content: [
          {
            type: "row",
            content: [
              {
                type: "component",
                title: "Test2",
                componentType: "Orange",
                minWidth: 5,
                minHeight: 100,
              },
              {
                type: "stack",
                content: [
                  {
                    type: "component",
                    title: "Editor",
                    componentType: "RemoteWindow",
                    componentState: {
                      serverType: "editor",
                    },
                  },
                  {
                    type: "component",
                    title: "Runtime",
                    componentType: "RemoteWindow",
                    componentState: {
                      serverType: "runtime",
                    },
                  },
                ],
              },
            ],
          },
          {
            type: "stack",
            height: 20,
            content: [
              {
                type: "component",
                componentType: "ResourceBrowser",
                title: "Resource Browser",
              },
              {
                type: "component",
                componentType: "LocalChanges",
                title: "Local Changes",
              },
              {
                type: "component",
                componentType: "Log",
              },
            ],
          },
        ],
      },
      {
        type: "component",
        componentType: "PropertyGrid",
        title: "Property Grid",
        width: 25,
      },
    ],
  },
};
