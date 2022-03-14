import type { Writable } from "svelte/store";
import { createContextMenuStore } from "@lgn/web-client/src/stores/contextMenu";
import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";

export type ContextMenuEntryRecord = {
  resource: { item: ResourceDescription | null; name: string };
  resourcePanel: { item: null; name: string };
};

export type ContextMenuValue = keyof ContextMenuEntryRecord;

export type ContextMenuStore = Writable<ContextMenuValue>;

export default createContextMenuStore<keyof ContextMenuEntryRecord>();
