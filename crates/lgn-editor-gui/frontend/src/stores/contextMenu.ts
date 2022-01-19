import buildContextMenuStore from "@lgn/frontend/src/stores/contextMenu";

export type ContextMenuEntryRecord = {
  resource: { itemName: string };
};

export default buildContextMenuStore<ContextMenuEntryRecord>();
