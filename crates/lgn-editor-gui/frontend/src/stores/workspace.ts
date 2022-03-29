import { createWorkspace } from "@lgn/web-client/src/stores/Workspace";

export type {
  WorkspaceValue,
  WorkspaceStore,
} from "@lgn/web-client/src/stores/workspace";

export const viewportPanelKey = "viewport";

export default createWorkspace();
