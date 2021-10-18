import { invoke } from "@tauri-apps/api/tauri";

function bytes_out(x) {
    return Buffer(JSON.stringify(x)).toString('base64');
}

function bytes_in(x) {
    return JSON.parse(Buffer.from(x, 'base64'));
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

export default function () { };