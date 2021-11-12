import { invoke } from "@tauri-apps/api/tauri";

import { bytes_in, bytes_out } from "./conversion";

export async function authenticate() {
    const result = await invoke("authenticate");

    return result;
}

export async function initialize_stream(localSessionDescription) {
    const result = await invoke("initialize_stream", {
        rtcSessionDescription: bytes_out(localSessionDescription.toJSON()),
    });

    return new RTCSessionDescription(bytes_in(result));
}

export async function search_resources() {
    return await invoke("search_resources");
};

export async function undo_transaction() {
    return await invoke("undo_transaction");
}

export async function redo_transaction() {
    return await invoke("redo_transaction")
}

export async function get_resource_properties(id) {
    var resp = await invoke("get_resource_properties", {
        request: {
            id: id,
        },
    });

    // We receive the `value` and `default_value` fields as a JSON strings.
    resp.properties.forEach(function (part, i, properties) {
        properties[i].value = JSON.parse(part.value);
        properties[i].default_value = JSON.parse(part.default_value);
    });

    return resp;
};

export async function update_resource_properties(id, version, propertyUpdates) {
    var resp = await invoke("update_resource_properties", {
        request: {
            id: id,
            version: version,
            property_updates: propertyUpdates.map(propertyUpdate => {
                propertyUpdate.value = JSON.stringify(propertyUpdate.value);

                return propertyUpdate;
            })
        },
    });

    // We receive the `value` and `default_value` fields as a JSON strings.
    resp.updated_properties.forEach(function (part, i, properties) {
        properties[i].value = JSON.parse(part.value);
    });

    return resp;
};

export function on_receive_control_message(json_msg) {
    return invoke("on_receive_control_message", { jsonMsg: json_msg });
}

export function on_send_edition_command(json_command) {
    return invoke("on_send_edition_command", { jsonCommand: json_command });
}

export function on_video_close() {
    return invoke("on_video_close");
}

export function on_video_chunk_received(chunkHeader) {
    return invoke("on_video_chunk_received", { chunkHeader: chunkHeader });
}

export default function () { };
