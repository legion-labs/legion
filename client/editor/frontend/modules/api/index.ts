// The types defined here are duplicated from the Rust code,
// we might need to use something like https://github.com/Wulf/tsync
// to improve the interop Rust <-> TS

import { invoke } from "@tauri-apps/api/tauri";

import { bytesIn, bytesOut } from "./conversion";

export function authenticate() {
  return invoke("authenticate");
}

export async function initializeStream(
  localSessionDescription: RTCSessionDescription
) {
  const result = await invoke<string>("initialize_stream", {
    rtcSessionDescription: bytesOut(localSessionDescription.toJSON()),
  });

  const rtcSessionDescriptionInit = bytesIn(
    result
  ) as RTCSessionDescriptionInit;

  return new RTCSessionDescription(rtcSessionDescriptionInit);
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

type JSResourceProperty = {
  name: string;
  ptype: string;
  default_value: string;
  value: string;
  group: string;
};

type JSResourceDescription = {
  id: string;
  path: string;
  version: number;
};

type JSGetResourcePropertiesResponse = {
  description: JSResourceDescription;
  properties: JSResourceProperty[];
};

export async function getResourceProperties(id: string) {
  const resp = await invoke<JSGetResourcePropertiesResponse>(
    "get_resource_properties",
    {
      request: {
        id,
      },
    }
  );

  // We receive the `value` and `default_value` fields as JSON strings.
  resp.properties.forEach(function (property) {
    property.value = JSON.parse(property.value);
    property.default_value = JSON.parse(property.default_value);
  });

  return resp;
}

type JSResourcePropertyUpdate = {
  name: string;
  value: string;
};

type JSUpdateResourcePropertiesResponse = {
  version: number;
  updated_properties: JSResourcePropertyUpdate[];
};

export async function updateResourceProperties(
  id: string,
  version: number,
  propertyUpdates: { name: string; value: unknown }[]
) {
  const resp = await invoke<JSUpdateResourcePropertiesResponse>(
    "update_resource_properties",
    {
      request: {
        id,
        version,
        property_updates: propertyUpdates.map((propertyUpdate) => {
          propertyUpdate.value = JSON.stringify(propertyUpdate.value);

          return propertyUpdate;
        }),
      },
    }
  );

  // We receive the `value` field as a JSON string.
  resp.updated_properties.forEach(function (property) {
    property.value = JSON.parse(property.value);
  });

  return resp;
}

export function onReceiveControlMessage(jsonMsg: string) {
  return invoke("on_receive_control_message", { jsonMsg });
}

export function onSendEditionCommand(jsonCommand: string) {
  return invoke("on_send_edition_command", { jsonCommand });
}

export function onVideoClose() {
  return invoke("on_video_close");
}

export function onVideoChunkReceived(chunkHeader: string) {
  return invoke("on_video_chunk_received", { chunkHeader });
}

// eslint-disable-next-line @typescript-eslint/no-empty-function
export default () => {};
