import ContextMenyStore from "@lgn/web-client/src/stores/contextMenu";
import { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";

export type ContextMenuEntryRecord = {
  resource: { item: ResourceDescription | null; name: string };
};

export default new ContextMenyStore<ContextMenuEntryRecord>();
