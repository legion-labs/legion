import { invoke } from "@tauri-apps/api/tauri";

import { bytes_in, bytes_out } from "./conversion";

export async function initialize_stream(localSessionDescription) {
    const result = await invoke("initialize_stream", {
        rtcSessionDescription: bytes_out(localSessionDescription.toJSON()),
    });

    return new RTCSessionDescription(bytes_in(result));
}

export async function search_resources() {
    var result = await invoke("search_resources");

    // TODO: Remove this once we have a more comprehensive data-set to test on.
    result.resource_descriptions = [
        {
            "id": "triangle",
            path: "fake/triangle",
            version: 1,
        },
        ...result.resource_descriptions,
    ];

    return result;
};

export async function get_resource_properties(id) {

    // TODO: Remove this once we have a more comprehensive data-set to test on.
    if (id == "triangle") {
        return {
            description: {
                "id": "triangle",
                path: "fake/triangle",
                version: 1,
            },
            properties: [
                {
                    name: "color",
                    ptype: "color",
                    value: "#FF0000FF",
                    default_value: "#FF0000FF",
                    group: "material",
                },
                {
                    name: "speed",
                    ptype: "speed",
                    value: 1,
                    default_value: 1,
                    group: "movement",
                },
            ],
        };
    }
    var resp = await invoke("get_resource_properties", { id: id });

    // We receive the `value` and `default_value` fields as a JSON-string bytes
    // array.
    resp.properties.forEach(function (part, i, properties) {
        properties[i].value = JSON.parse(String.fromCharCode.apply(String, part.value));
        properties[i].default_value = JSON.parse(String.fromCharCode.apply(String, part.default_value));
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
