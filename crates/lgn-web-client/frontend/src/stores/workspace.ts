// This module contains and orchestrator all the tiles and panels, their content and status.
// In the future it will also handle their size and position.
import type { Readable } from "svelte/store";
import { writable } from "svelte/store";

import type { NonEmptyArray } from "../lib/array";
import { isNonEmpty } from "../lib/array";
import type { Size } from "../lib/types";

/**
 * Defining the tab behavior and content is up to the developer and will differ depending
 * on the application, though, a set of common attributes is expected to be present in all tabs.
 */
export type TabTypeBase = {
  /**
   * An arbitrary id, random or not, that will help identifying the tab or linking it
   * with external resources
   */
  id: string;
  /** The displayed label of the tab */
  label: string;
  /**
   * Whether or not the pushed tab can be "removed" from the store.
   * This config might get removed later on as all tab will be disposable.
   */
  disposable?: boolean;
  payloadId?: string;
};

// TODO: push size and position to the panel
/**
 * A Panel is responsible for its own tabs and contents.
 */
export type Panel<Tab extends TabTypeBase> =
  | { type: "emptyPanel"; id: string }
  | {
      type: "populatedPanel";
      id: string;
      tabs: NonEmptyArray<Tab>;
      activeTabIndex: number;
    };

export function createEmptyPanel<Tab extends TabTypeBase>(
  id: string
): Panel<Tab> {
  return { type: "emptyPanel", id };
}

export function createPanel<Tab extends TabTypeBase>(
  id: string,
  tabs: NonEmptyArray<Tab>,
  { activeTabIndex = 0 }: { activeTabIndex?: number } = {}
): Panel<Tab> {
  return {
    type: "populatedPanel",
    id,
    tabs,
    activeTabIndex,
  };
}

export type TileSize =
  | { type: "tracked"; value: Size | null }
  | { type: "untracked" };

/**
 * A Tile is a basic UI element that doesn't handle anything but its size and position.
 * Typically a Tile contains one Panel.
 */
export type Tile<Tab extends TabTypeBase> = {
  id: string;
  panel: Panel<Tab> | null;
  size: TileSize;
};

export function createTile<Tab extends TabTypeBase>(
  id: string,
  panel: Panel<Tab>,
  { trackSize = false }: { trackSize?: boolean } = {}
): Tile<Tab> {
  return {
    id,
    panel,
    size: trackSize ? { type: "tracked", value: null } : { type: "untracked" },
  };
}

export type Workspace<Tab extends TabTypeBase> = {
  tiles: Tile<Tab>[];
};

export type WorkspaceValue<Tab extends TabTypeBase> = Workspace<Tab>;

export type WorkspaceStore<Tab extends TabTypeBase> = Readable<
  WorkspaceValue<Tab>
> & {
  // Tile
  pushTile(tile: Tile<Tab>): void;
  appendAllTiles(tiles: NonEmptyArray<Tile<Tab>>): void;
  removeTile(tileId: string): void;
  removeTileByValue(tile: Tile<Tab>): void;
  setPanelToTile(tileId: string, panel: Panel<Tab>): void;

  // Panel
  /** To remove a panel will _not_ remove the tile it belongs to, use `removeTile` for that */
  removePanel(panelId: string): void;
  /** To remove a panel will _not_ remove the tile it belongs to, use `removeTileByValue` for that */
  removePanelByValue(panel: Panel<Tab>): void;

  // Tab
  pushTabToPanel(
    panelId: string,
    tab: Tab,
    config?: {
      /**
       * Focus the newly pushed tab.
       */
      focus?: boolean;
    }
  ): void;
  pushAllTabsToPanel(panelId: string, tabs: NonEmptyArray<Tab>): void;
  setPanelTabs(panelId: string, tabs: NonEmptyArray<Tab> | null): void;
  removeTabFromPanel(panelId: string, tabId: string): void;
  removeTabFromPanelByValue(panelId: string, tab: Tab): void;
  activateTabInPanel(panelId: string, tabId: string): void;
  activateTabInPanelByValue(panelId: string, tab: Tab): void;
};

export function createWorkspace<Tab extends TabTypeBase>(
  initialTiles: Tile<Tab>[] = []
): WorkspaceStore<Tab> {
  const workspace = writable<Workspace<Tab>>({ tiles: initialTiles });

  function updatePanel(
    panelId: string,
    update: (panel: Panel<Tab>) => Panel<Tab>
  ) {
    workspace.update((workspace) => ({
      ...workspace,
      tiles: workspace.tiles.map((tile) => {
        if (tile.panel?.id !== panelId) {
          return tile;
        }

        return { ...tile, panel: update(tile.panel) };
      }),
    }));
  }

  return {
    ...workspace,

    pushTile(tile) {
      workspace.update((workspace) =>
        workspace.tiles.some(({ id }) => id === tile.id)
          ? workspace
          : {
              ...workspace,
              tiles: [...workspace.tiles, tile],
            }
      );
    },

    appendAllTiles(newTiles) {
      workspace.update((workspace) => ({
        ...workspace,
        tiles: [
          ...workspace.tiles,
          ...newTiles.filter(
            (newTile) => !workspace.tiles.some((tile) => tile.id === newTile.id)
          ),
        ],
      }));
    },

    removeTile(tileId) {
      workspace.update((workspace) => ({
        ...workspace,
        tiles: workspace.tiles.filter((tile) => tile.id !== tileId),
      }));
    },

    removeTileByValue(tile) {
      workspace.update((workspace) => ({
        ...workspace,
        tiles: workspace.tiles.filter((value) => value !== tile),
      }));
    },

    setPanelToTile(tileId, panel) {
      workspace.update((workspace) => ({
        ...workspace,
        tiles: workspace.tiles.map((tile) =>
          tile.id === tileId ? { ...tile, panel } : tile
        ),
      }));
    },

    removePanel(panelId) {
      workspace.update((workspace) => ({
        ...workspace,
        tiles: workspace.tiles.map((tile) =>
          tile.panel?.id === panelId ? { ...tile, panel: null } : tile
        ),
      }));
    },

    removePanelByValue(panel) {
      workspace.update((workspace) => ({
        ...workspace,
        tiles: workspace.tiles.map((tile) =>
          tile.panel === panel ? { ...tile, panel: null } : tile
        ),
      }));
    },

    pushTabToPanel(panelId, tab, { focus } = { focus: false }) {
      updatePanel(panelId, (panel) =>
        panel.type === "emptyPanel"
          ? {
              ...panel,
              type: "populatedPanel",
              tabs: [tab],
              activeTabIndex: 0,
            }
          : panel.tabs.some(({ id }) => id === tab.id)
          ? panel
          : {
              ...panel,
              tabs: [...panel.tabs, tab],
              activeTabIndex: focus ? panel.tabs.length : panel.activeTabIndex,
            }
      );
    },

    pushAllTabsToPanel(panelId, tabs) {
      updatePanel(panelId, (panel) =>
        panel.type === "emptyPanel"
          ? {
              ...panel,
              type: "populatedPanel",
              tabs,
              activeTabIndex: 0,
            }
          : {
              ...panel,
              tabs: [
                ...panel.tabs,
                ...tabs.filter(
                  (tab) => !panel.tabs.some(({ id }) => id === tab.id)
                ),
              ],
            }
      );
    },

    setPanelTabs(panelId: string, tabs: NonEmptyArray<Tab> | null) {
      updatePanel(panelId, (panel) => {
        if (!tabs) {
          return { id: panel.id, type: "emptyPanel" };
        }

        return {
          ...panel,
          type: "populatedPanel",
          activeTabIndex: 0,
          tabs,
        };
      });
    },

    removeTabFromPanel(panelId, tabId) {
      updatePanel(panelId, (panel) => {
        if (panel.type === "emptyPanel") {
          return panel;
        }

        const updatedPanelTabs = panel.tabs.filter((tab) => tab.id !== tabId);

        if (updatedPanelTabs.length === panel.tabs.length) {
          return panel;
        }

        if (!isNonEmpty(updatedPanelTabs)) {
          return { id: panel.id, type: "emptyPanel" };
        }

        const activeTabIndex =
          panel.activeTabIndex in updatedPanelTabs ? panel.activeTabIndex : 0;

        return { ...panel, activeTabIndex, tabs: updatedPanelTabs };
      });
    },

    removeTabFromPanelByValue(panelId, tab) {
      updatePanel(panelId, (panel) => {
        if (panel.type === "emptyPanel") {
          return panel;
        }

        const updatedPanelTabs = panel.tabs.filter(
          (panelTab) => panelTab !== tab
        );

        if (updatedPanelTabs.length === panel.tabs.length) {
          return panel;
        }

        if (!isNonEmpty(updatedPanelTabs)) {
          return { id: panel.id, type: "emptyPanel" };
        }

        const activeTabIndex =
          panel.activeTabIndex in updatedPanelTabs ? panel.activeTabIndex : 0;

        return { ...panel, activeTabIndex, tabs: updatedPanelTabs };
      });
    },

    activateTabInPanel(panelId, tabId) {
      updatePanel(panelId, (panel) => {
        if (panel.type === "emptyPanel") {
          return panel;
        }

        const activeTabIndex = panel.tabs.findIndex((tab) => tab.id === tabId);

        return {
          ...panel,
          activeTabIndex:
            activeTabIndex >= 0 ? activeTabIndex : panel.activeTabIndex,
        };
      });
    },

    activateTabInPanelByValue(panelId, tab) {
      updatePanel(panelId, (panel) => {
        if (panel.type === "emptyPanel") {
          return panel;
        }

        const activeTabIndex = panel.tabs.findIndex(
          (panelTab) => panelTab === tab
        );

        return {
          ...panel,
          activeTabIndex:
            activeTabIndex >= 0 ? activeTabIndex : panel.activeTabIndex,
        };
      });
    },
  };
}
