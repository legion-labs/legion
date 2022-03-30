import type { TabTypeBase } from "@lgn/web-client/src/stores/workspace";
import { createWorkspace } from "@lgn/web-client/src/stores/workspace";
import type {
  WorkspaceStore as WorkspaceStoreBase,
  WorkspaceValue as WorkspaceValueBase,
} from "@lgn/web-client/src/stores/workspace";

/** Monaco script tabs */
export type ScriptTabType = {
  type: "script";
  payloadId: string;
};

/** The video/player viewport */
export type VideoTabType = {
  type: "video";
  payloadId: string;
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

export type WorkspaceValue = WorkspaceValueBase<TabType>;

export type WorkspaceStore = WorkspaceStoreBase<TabType>;

export const viewportTileId = "viewport-tile";

export const viewportPanelId = "viewport-panel";

export default createWorkspace<TabType>();
