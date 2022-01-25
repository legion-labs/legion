import buildContextMenu from "@lgn/frontend/src/actions/contextMenu";
import contextMenuStore, { ContextMenuEntryRecord } from "@/stores/contextMenu";

export default buildContextMenu<ContextMenuEntryRecord>(contextMenuStore);
