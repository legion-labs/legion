import { Streaming } from "@lgn/apis";

import { blobToJson, jsonToBlob } from "../lib/api";
import { addAuthToClient } from "../lib/client";
import log from "../lib/log";

const defaultRestEditorServerUrl = "http://[::1]:5051";
const defaultRestRuntimeServerUrl = "http://[::1]:5052";

export type ServerType = "editor" | "runtime";

let editorClient: Streaming.Client;

let runtimeClient: Streaming.Client;

function getClientFor(type: ServerType): Streaming.Client {
  switch (type) {
    case "editor":
      return editorClient;

    case "runtime":
      return runtimeClient;
  }
}

export function initApiClient({
  restEditorServerUrl = defaultRestEditorServerUrl,
  restRuntimeServerUrl = defaultRestRuntimeServerUrl,
  accessTokenCookieName,
  fetch,
}: {
  restEditorServerUrl?: string;
  restRuntimeServerUrl?: string;
  accessTokenCookieName?: string;
  fetch?: typeof globalThis.fetch;
} = {}) {
  if (accessTokenCookieName !== undefined) {
    editorClient = addAuthToClient(
      new Streaming.Client({
        baseUri: restEditorServerUrl,
        fetch,
      }),
      accessTokenCookieName
    );

    runtimeClient = addAuthToClient(
      new Streaming.Client({
        baseUri: restRuntimeServerUrl,
        fetch,
      }),
      accessTokenCookieName
    );
  } else {
    editorClient = new Streaming.Client({
      baseUri: restEditorServerUrl,
      fetch,
    });

    runtimeClient = new Streaming.Client({
      baseUri: restRuntimeServerUrl,
      fetch,
    });
  }
}

/**
 * Initialize the video player stream
 * @param serverType
 * @param localSessionDescription
 * @returns a valid RTC sessions description to use with an RTCPeerConnection
 */
export async function initializeStream(
  serverType: ServerType,
  localSessionDescription: RTCSessionDescription
) {
  const client = getClientFor(serverType);

  const response = await client.initializeStream({
    params: { "space-id": "0", "workspace-id": "0" },
    // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
    body: jsonToBlob(localSessionDescription.toJSON()),
  });

  // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
  return new RTCSessionDescription(await blobToJson(response.value));
}

/**
 * Used for logging purpose
 * @param jsonMsg
 * @returns
 */
export function onReceiveControlMessage(jsonMsg: string) {
  log.info("video", `Received control message. msg=${jsonMsg}`);
}

// TODO: Implement logging and telemetry (https://github.com/legion-labs/legion/issues/481)
/**
 * Used for logging and telemetry purpose
 * @param _chunkHeader
 * @returns
 */
export function onVideoChunkReceived(_chunkHeader: string) {
  return;
}

// TODO: Implement logging and telemetry (https://github.com/legion-labs/legion/issues/481)
/**
 * Used for logging and telemetry purpose
 * @returns
 */
export function onVideoClose() {
  return;
}
