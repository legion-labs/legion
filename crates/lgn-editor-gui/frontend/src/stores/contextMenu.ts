import buildContextMenuStore from "@lgn/frontend/src/stores/contextMenu";
import { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";

export type ContextMenuEntryRecord = {
  resource: { item: ResourceDescription | null; name: string };
};

export default buildContextMenuStore<ContextMenuEntryRecord>();
