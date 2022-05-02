import type { ResourceDescription } from "@lgn/proto-editor/dist/resource_browser";

import type { Entry } from "./hierarchyTree";

// const offlineEntity = "|1c0ff9e497b0740f";
const runtimeEntity = "|1d9ddd99aad89045";
// const offlineScript = "|45307156cdfef705";
const runtimeScript = "|17bed132bc803773";
const offlineMaterial = "|07dd9f5d1793ed64";
const runtimeMaterial = "|738a5b850f6f8b3a";
// const offlineMesh = "|80b97dd7e2e9edca";
// const runtimeMesh = "|9c9a840b84935d1f";
// const offlinePsd = "|4976b194d8a1e0c2";
// const png = "|1a377760c95e361c";
const offlineTexture = "|648242a8cf00bfee";
const runtimeTexture = "|0659c0b6c23b950d";
const offlineModel = "|44e4b6023fb7a8d3";
const runtimeModel = "|5c4b1b522bf5dcb0";

export function createResourcePathId(
  resourceType: string,
  entry: Entry<ResourceDescription>
): string | null {
  let result = `${entry.item.id}`;

  switch (resourceType.toLowerCase()) {
    case "model": {
      switch (entry.item.type) {
        case "model":
          break;
        case "gltfzip":
        case "gltf":
        case "glb":
          result += offlineModel;
          result += "_Model";
          break;
        default:
          return null;
      }
      result += runtimeModel;
      break;
    }

    case "material": {
      switch (entry.item.type) {
        case "material":
          break;
        case "gltfzip":
        case "gltf":
        case "glb":
          result += offlineMaterial + "_Material";
          break;
        default:
          return null;
      }
      result += runtimeMaterial;
      break;
    }

    case "entity": {
      switch (entry.item.type) {
        case "entity":
          break;
        default:
          return null;
      }
      result += runtimeEntity;
      break;
    }

    case "texture": {
      switch (entry.item.type) {
        case "texture":
          break;
        case "psd":
          result += offlineTexture;
          break;
        case "png":
          result += offlineTexture;
          break;
        case "gltfzip":
        case "gltf":
        case "glb":
          result += offlineTexture + "_0";
          break;
        default:
          return null;
      }
      result += runtimeTexture + "_Albedo";
      break;
    }

    case "script": {
      switch (entry.item.type) {
        case "script":
          break;
        default:
          return null;
      }
      result += runtimeScript;
      break;
    }
  }

  return result;
}

export function iconFor(entry: Entry<ResourceDescription | symbol>) {
  if (typeof entry.item === "symbol") {
    return "ic:baseline-folder-open";
  }

  switch (entry.item.type) {
    case "entity": {
      return "ic:outline-token";
    }

    case "script": {
      return "ic:outline-text-snippet";
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
    case "gltfzip":
    case "gltf":
    case "glb": {
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
