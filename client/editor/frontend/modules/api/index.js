import { invoke } from "@tauri-apps/api/tauri";

import { bytes_in, bytes_out } from "./conversion";

export async function initialize_stream(localSessionDescription) {
    const result = await invoke("initialize_stream", {
        rtcSessionDescription: bytes_out(localSessionDescription.toJSON()),
    });

    return new RTCSessionDescription(bytes_in(result));
}

export async function search_resources() {
    return await invoke("search_resources");
};

export async function get_resource_properties(id) {
    return await invoke("get_resource_properties", { id: id });
};
export default function () { };