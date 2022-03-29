// This module contains and orchestrator all the panels, their status, and their content.
// In the future it will also handle their size and position.
import type { Readable } from "svelte/store";
import { writable } from "svelte/store";

export type TabTypeBase = {
  name: string;
  /**
   * Whether or not the added tab can be "removed" from the store.
   * This config might get removed later on as all tab will be "removable" to an extent.
   */
  removable?: boolean;
};

/** Monaco script tabs */
export type ScriptTabType = {
  type: "script";
  getValue(): string;
  onChange(newValue: string): void;
  readonly?: boolean;
  lang: string;
};

/** The video/player viewport */
export type VideoTabType = {
  type: "video";
};

/** The resource control */
export type ResourceControlTabType = {
  type: "resourceControl";
};

/** The log */
export type LogTabType = {
  type: "log";
};

/** The property grid */
export type PropertyGridTabType = {
  type: "propertyGrid";
};

/** The scene explorer */
export type SceneExplorerTabType = {
  type: "ceneExplorer";
};

/** The resource browser */
export type ResourceBrowserTabType = {
  type: "resourceBrowser";
};

export type TabType = TabTypeBase &
  (
    | ScriptTabType
    | VideoTabType
    | ResourceControlTabType
    | LogTabType
    | PropertyGridTabType
    | SceneExplorerTabType
    | ResourceBrowserTabType
  );

/** Used when adding a new tab to a panel */
export type AddTabConfig = {
  /**
   * Focus the newly added tab.
   */
  focus?: boolean;
};

// TODO: Add size and position to the panel
export type Panel<Tab extends TabType = TabType> = {
  tabs: Map<symbol, Tab>;
  activeTab: Tab | null;
};

export function createPanel<Tab extends TabType = TabType>(
  initialTabs: Map<symbol, Tab> = new Map(),
  activeTab: Tab | null = null
): Panel<Tab> {
  return { tabs: initialTabs, activeTab };
}

export type WorkspaceValue<Tab extends TabType = TabType> = Record<
  string,
  Panel<Tab> | undefined
>;

export type WorkspaceStore<Tab extends TabType = TabType> =
  Readable<WorkspaceValue> & {
    // Workspace related
    addPanel(key: string, panel: Panel<Tab>): void;
    addAllPanels(newPanels: Record<string, Panel<Tab>>): void;
    removePanel(key: string): void;
    removePanelByValue(panel: Panel<Tab>): void;

    // Panel related
    addTab(
      panelKey: string,
      key: symbol,
      tab: Tab,
      config?: AddTabConfig
    ): void;
    addAllTabs(panelKey: string, tabs: Map<symbol, Tab>): void;
    removeTab(panelKey: string, key: symbol): void;
    removeTabByValue(panelKey: string, tab: Tab): void;
    activateTab(panelKey: string, key: symbol): void;
    activateTabByValue(panelKey: string, tab: Tab): void;
  };

export function createWorkspace<Tab extends TabType = TabType>(
  initialPanels: Record<string, Panel<Tab>> = {}
): WorkspaceStore<Tab> {
  const panels = writable(initialPanels);

  function updatePanel(
    panelKey: string,
    update: (panel: Panel<Tab>) => Panel<Tab>
  ) {
    panels.update((panels) => {
      const panel = panels[panelKey];

      if (!panel) {
        return panels;
      }

      const newPanel = update(panel);

      if (newPanel === panel) {
        return panels;
      }

      return {
        ...panels,
        [panelKey]: newPanel,
      };
    });
  }

  return {
    ...panels,

    addPanel(key, panel) {
      panels.update((panels) => ({ ...panels, [key]: panel }));
    },

    addAllPanels(newPanels) {
      panels.update((panels) => ({ ...panels, ...newPanels }));
    },

    removePanel(key: string) {
      panels.update((panels) => {
        const { [key]: removedTab, ...remainingPanels } = panels;

        if (!removedTab) {
          return panels;
        }

        return remainingPanels;
      });
    },

    removePanelByValue(panel) {
      panels.update((panels) => {
        let foundKey: string | null = null;

        for (const [key, value] of Object.entries(panels)) {
          if (value === panel) {
            foundKey = key;

            break;
          }
        }

        if (!foundKey) {
          return panels;
        }

        const { [foundKey]: removedTab, ...remainingPanels } = panels;

        if (!removedTab) {
          return panels;
        }

        return remainingPanels;
      });
    },

    addTab(panelKey, key, tab, { focus } = { focus: false }) {
      updatePanel(panelKey, (panel) => {
        panel.tabs.set(key, tab);

        if (focus) {
          panel.activeTab = tab;
        }

        return panel;
      });
    },

    addAllTabs(panelKey, tabs) {
      updatePanel(panelKey, (panel) => {
        panel.tabs = new Map([...panel.tabs, ...tabs]);

        return panel;
      });
    },

    removeTab(panelKey, key) {
      updatePanel(panelKey, (panel) => {
        if (!panel.tabs.has(key)) {
          return panel;
        }

        const tabToRemove = panel.tabs.get(key);

        const tabToRemoveIsActive = tabToRemove === panel.activeTab;

        const removed = panel.tabs.delete(key);

        // We select the first tab (if possible) if the removed tab was active
        if (removed && tabToRemoveIsActive) {
          const tab = panel.tabs.values().next();

          // If the tab map contains no value (and is therefore empty)
          // we can just set the active tab to "null"
          panel.activeTab = tab.done ? null : tab.value;
        }

        return panel;
      });
    },

    removeTabByValue(panelKey, tab) {
      updatePanel(panelKey, (panel) => {
        let foundKey: symbol | null = null;

        for (const [key, value] of panel.tabs) {
          if (value === tab) {
            foundKey = key;

            break;
          }
        }

        if (!foundKey) {
          return panel;
        }

        if (!panel.tabs.has(foundKey)) {
          return panel;
        }

        const tabToRemove = panel.tabs.get(foundKey);

        const tabToRemoveIsActive = tabToRemove === panel.activeTab;

        const removed = panel.tabs.delete(foundKey);

        // We select the first tab (if possible) if the removed tab was active
        if (removed && tabToRemoveIsActive) {
          const tab = panel.tabs.values().next();

          // If the tab map contains no value (and is therefore empty)
          // we can just set the active tab to "null"
          panel.activeTab = tab.done ? null : tab.value;
        }

        return panel;
      });
    },

    activateTab(panelKey, key) {
      updatePanel(panelKey, (panel) => {
        const activeTab = panel.tabs.get(key);

        if (!activeTab) {
          return panel;
        }

        panel.activeTab = activeTab;

        return panel;
      });
    },

    activateTabByValue(panelKey, tab) {
      updatePanel(panelKey, (panel) => {
        let foundKey: symbol | null = null;

        for (const [key, value] of panel.tabs) {
          if (value === tab) {
            foundKey = key;

            break;
          }
        }

        if (!foundKey) {
          return panel;
        }

        const activeTab = panel.tabs.get(foundKey);

        if (!activeTab) {
          return panel;
        }

        panel.activeTab = activeTab;

        return panel;
      });
    },
  };
}
