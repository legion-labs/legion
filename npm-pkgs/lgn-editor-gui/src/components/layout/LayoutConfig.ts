import type { LayoutConfig } from "golden-layout";
import type { SvelteComponentDev } from "svelte/internal";

import RemoteWindowSvelte from "@lgn/web-client/src/components/RemoteWindow.svelte";
import ScriptEditor from "@lgn/web-client/src/components/ScriptEditor.svelte";

import Log from "../Log.svelte";
import ResourceBrowser from "../ResourceBrowser.svelte";
import SceneExplorer from "../SceneExplorer.svelte";
import LocalChanges from "../localChanges/LocalChanges.svelte";
import PropertyGrid from "../propertyGrid/PropertyGrid.svelte";

export const AppComponentMap: Record<string, typeof SvelteComponentDev> = {
  PropertyGrid: PropertyGrid,
  ResourceBrowser: ResourceBrowser,
  SceneExplorer: SceneExplorer,
  RemoteWindow: RemoteWindowSvelte,
  LocalChanges: LocalChanges,
  Log: Log,
  ScriptEditor: ScriptEditor,
};

export const defaultLayoutConfig: LayoutConfig = {
  settings: {
    showPopoutIcon: false,
    tabControlOffset: 30,
  },
  dimensions: {
    minItemHeight: 100,
    minItemWidth: 200,
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
                type: "stack",
                content: [
                  {
                    type: "component",
                    title: "Editor",
                    id: "Editor",
                    componentType: "RemoteWindow",
                    componentState: {
                      state: {
                        serverType: "editor",
                      },
                    },
                  },
                  {
                    type: "component",
                    title: "Runtime",
                    id: "Runtime",
                    componentType: "RemoteWindow",
                    componentState: {
                      state: {
                        serverType: "runtime",
                      },
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
                id: "ResourceBrowser",
                componentType: "ResourceBrowser",
                title: "Resource Browser",
              },
              {
                type: "component",
                id: "LocalChanges",
                componentType: "LocalChanges",
                title: "Local Changes",
              },
              {
                type: "component",
                id: "Log",
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
        width: 10,
      },
    ],
  },
};
