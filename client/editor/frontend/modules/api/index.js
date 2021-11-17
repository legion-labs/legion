import { invoke } from "@tauri-apps/api/tauri";

import { bytesIn, bytesOut } from "./conversion";

export function authenticate() {
  return invoke("authenticate");
}

export async function initializeStream(localSessionDescription) {
  const result = await invoke("initialize_stream", {
    rtcSessionDescription: bytesOut(localSessionDescription.toJSON()),
  });

  return new RTCSessionDescription(bytesIn(result));
}

export function searchResources() {
  return invoke("search_resources");
}

export function undoTransaction() {
  return invoke("undo_transaction");
}

export function redoTransaction() {
  return invoke("redo_transaction");
}

export async function getResourceProperties(id) {
  const resp = await invoke("get_resource_properties", {
    request: {
      id,
    },
  });

  // We receive the `value` and `default_value` fields as a JSON strings.
  resp.properties.forEach(function (part, i, properties) {
    properties[i].value = JSON.parse(part.value);
    properties[i].default_value = JSON.parse(part.default_value);
  });

  return resp;
}

export async function updateResourceProperties(id, version, propertyUpdates) {
  const resp = await invoke("update_resource_properties", {
    request: {
      id,
      version,
      property_updates: propertyUpdates.map((propertyUpdate) => {
        propertyUpdate.value = JSON.stringify(propertyUpdate.value);

        return propertyUpdate;
      }),
    },
  });

  // We receive the `value` and `default_value` fields as a JSON strings.
  resp.updated_properties.forEach(function (part, i, properties) {
    properties[i].value = JSON.parse(part.value);
  });

  return resp;
}

export function onReceiveControlMessage(jsonMsg) {
  return invoke("on_receive_control_message", { jsonMsg });
}

export function onSendEditionCommand(jsonCommand) {
  return invoke("on_send_edition_command", { jsonCommand });
}

export function onVideoClose() {
  return invoke("on_video_close");
}

export function onVideoChunkReceived(chunkHeader) {
  return invoke("on_video_chunk_received", { chunkHeader });
}

export default () => {};
