import { ContextMenuEntryRecord } from "@/stores/contextMenu";
import { Entry } from "@lgn/frontend/src/stores/contextMenu";

const entries: Entry<ContextMenuEntryRecord["resource"]>[] = [
  {
    type: "item",
    label: "Rename",
    onClick({ close }) {
      close();
    },
  },
  {
    type: "item",
    label: "Clone",
    onClick({ close }) {
      close();
    },
  },
  {
    type: "item",
    label: "Delete",
    tag: "danger",
    onClick({ close }) {
      close();
    },
  },
  { type: "separator" },
  {
    type: "item",
    label: "Create new...",
    onClick({ close }) {
      close();
    },
  },
  {
    type: "item",
    label: "Import...",
    onClick({ close }) {
      close();
    },
  },
  { type: "separator" },
  {
    type: "item",
    label: "Help",
    onClick({ close }) {
      close();
    },
  },
];

export default entries;
