import { Entry } from "@lgn/frontend/src/types/contextMenu";

const entries: Entry[] = [
  { type: "item", action: "rename", label: "Rename" },
  { type: "item", action: "clone", label: "Clone" },
  { type: "item", action: "remove", label: "Delete", tag: "danger" },
  { type: "separator" },
  { type: "item", action: "create-new", label: "Create new..." },
  { type: "item", action: "import", label: "Import..." },
  { type: "separator" },
  { type: "item", action: "help", label: "Help" },
];

export default entries;
