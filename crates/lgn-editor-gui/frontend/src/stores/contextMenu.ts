import type { Writable } from "svelte/store";

import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
import { createContextMenuStore } from "@lgn/web-client/src/stores/contextMenu";

export type ContextMenuEntryRecord = {
  resource: { item: ResourceDescription | null; name: string };
  resourcePanel: { item: null; name: string };
};

export type ContextMenuValue = keyof ContextMenuEntryRecord;

export type ContextMenuStore = Writable<ContextMenuValue>;

export default createContextMenuStore<keyof ContextMenuEntryRecord>();
