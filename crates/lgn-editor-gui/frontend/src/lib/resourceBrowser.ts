import { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";
import { Entry } from "./hierarchyTree";

export function iconFor(entry: Entry<ResourceDescription>) {
  switch (entry.item.type) {
    case "entity": {
      return "ic:outline-token";
    }

    case "script": {
      return "ic:outline-text_snippet";
    }

    case "instance": {
      return "ic:outline-pages";
    }

    case "material": {
      return "ic:outline-style";
    }

    case "mesh":
    case "model":

    // eslint-disable-next-line no-fallthrough
    case "gltf": {
      return "ic:outline-format-shapes";
    }

    case "psd":
    case "png":

    // eslint-disable-next-line no-fallthrough
    case "texture": {
      return "ic:outline-image";
    }
  }

  return entry.subEntries.length
    ? "ic:baseline-folder-open"
    : "ic:outline-insert-drive-file";
}
