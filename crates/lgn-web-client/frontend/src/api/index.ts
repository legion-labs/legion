import {
  StreamerClientImpl,
  GrpcWebImpl as StreamingGrpcWebImpl,
} from "@lgn/proto-streaming/dist/streaming";
import { bytesToJson, jsonToBytes } from "../lib/api";
import log from "../lib/log";

// TODO: Move to config
const defaultEditorServerUrl = "http://[::1]:50051";
const defaultRuntimeServerUrl = "http://[::1]:50052";

export type ServerType = "editor" | "runtime";

let editorClient: StreamerClientImpl;

let runtimeClient: StreamerClientImpl;

function getClientFor(type: ServerType): StreamerClientImpl {
  switch (type) {
    case "editor":
      return editorClient;

    case "runtime":
      return runtimeClient;
  }
}

export function initApiClient({
  editorServerUrl = defaultEditorServerUrl,
  runtimeServerUrl = defaultRuntimeServerUrl,
}: {
  editorServerUrl?: string;
  runtimeServerUrl?: string;
} = {}) {
  editorClient = new StreamerClientImpl(
    new StreamingGrpcWebImpl(editorServerUrl, {
      debug: false,
    })
  );

  runtimeClient = new StreamerClientImpl(
    new StreamingGrpcWebImpl(runtimeServerUrl, {
      debug: false,
    })
  );
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
    rtcSessionDescription: jsonToBytes(localSessionDescription.toJSON()),
  });

  return new RTCSessionDescription(bytesToJson(response.rtcSessionDescription));
}

/**
 * Used for logging purpose
 * @param jsonMsg
 * @returns
 */
export async function onReceiveControlMessage(jsonMsg: string) {
  log.info("video", `Received control message. msg=${jsonMsg}`);
}

// TODO: Implement logging and telemetry (https://github.com/legion-labs/legion/issues/481)
/**
 * Used for logging and telemetry purpose
 * @param _chunkHeader
 * @returns
 */
export async function onVideoChunkReceived(_chunkHeader: string) {
  return;
}

// TODO: Implement logging and telemetry (https://github.com/legion-labs/legion/issues/481)
/**
 * Used for logging and telemetry purpose
 * @returns
 */
export async function onVideoClose() {
  return;
}
