import type { Writable } from "svelte/store";

import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
import { createContextMenuStore } from "@lgn/web-client/src/stores/contextMenu";
import type { Entry } from "@lgn/web-client/src/types/contextMenu";

export const resourceBrowserItemContextMenuId =
  "resourceBrowserItemContextMenu";

export const resourceBrowserItemEntries: Entry[] = [
  { type: "item", action: "rename", label: "Rename" },
  { type: "item", action: "clone", label: "Clone" },
  { type: "item", action: "remove", label: "Delete", tag: "danger" },
  { type: "separator" },
  { type: "item", action: "new", label: "Create new..." },
  { type: "item", action: "import", label: "Import..." },
  { type: "item", action: "openScene", label: "Open Scene..." },
  { type: "separator" },
  { type: "item", action: "help", label: "Help" },
];

export const resourceBrowserPanelContextMenuId =
  "resourceBrowserPanelContextMenu";

export const resourceBrowserPanelEntries: Entry[] = [
  { type: "item", action: "new", label: "Create new..." },
  { type: "item", action: "import", label: "Import..." },
  { type: "separator" },
  { type: "item", action: "help", label: "Help" },
];

export type ContextMenuEntryRecord = {
  [resourceBrowserItemContextMenuId]: {
    item: ResourceDescription | null;
    name: string;
  };
  [resourceBrowserPanelContextMenuId]: { item: null; name: string };
};

export type ContextMenuValue = keyof ContextMenuEntryRecord;

export type ContextMenuStore = Writable<ContextMenuValue>;

export default createContextMenuStore<keyof ContextMenuEntryRecord>();
