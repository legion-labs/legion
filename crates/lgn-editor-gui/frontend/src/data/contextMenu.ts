import type { Entry } from "@lgn/web-client/src/types/contextMenu";

export const resourceEntries: Entry[] = [
  { type: "item", action: "rename", label: "Rename" },
  { type: "item", action: "clone", label: "Clone" },
  { type: "item", action: "remove", label: "Delete", tag: "danger" },
  { type: "separator" },
  { type: "item", action: "new", label: "Create new..." },
  { type: "item", action: "import", label: "Import..." },
  { type: "item", action: "open_scene", label: "Open Scene..." },
  { type: "separator" },
  { type: "item", action: "sync_latest", label: "Sync Latest" },
  { type: "item", action: "commit", label: "Commit Changes" },
  { type: "separator" },
  { type: "item", action: "help", label: "Help" },
];

export const resourcePanelEntries: Entry[] = [
  { type: "item", action: "new", label: "Create new..." },
  { type: "item", action: "import", label: "Import..." },
  { type: "separator" },
  { type: "item", action: "sync_latest", label: "Sync Latest" },
  { type: "item", action: "commit", label: "Commit Changes" },
  { type: "separator" },
  { type: "item", action: "help", label: "Help" },
];
