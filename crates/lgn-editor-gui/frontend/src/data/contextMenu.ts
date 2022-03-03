import type { Entry } from "@lgn/web-client/src/types/contextMenu";

export const resourceEntries: Entry[] = [
  { type: "item", action: "rename", label: "Rename" },
  { type: "item", action: "clone", label: "Clone" },
  { type: "item", action: "remove", label: "Delete", tag: "danger" },
  { type: "separator" },
  { type: "item", action: "new", label: "Create new..." },
  { type: "item", action: "import", label: "Import..." },
  { type: "separator" },
  { type: "item", action: "help", label: "Help" },
];

export const resourcePanelEntries: Entry[] = [
  { type: "item", action: "new", label: "Create new..." },
  { type: "item", action: "import", label: "Import..." },
  { type: "separator" },
  { type: "item", action: "help", label: "Help" },
];
