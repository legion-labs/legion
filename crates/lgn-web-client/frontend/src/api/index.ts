import {
  StreamerClientImpl,
  GrpcWebImpl as StreamingGrpcWebImpl,
} from "@lgn/proto-streaming/dist/streaming";
import { bytesToJson, jsonToBytes } from "../lib/api";
import log from "../lib/log";

// TODO: Move to config
const editorServerURL = "http://[::1]:50051";
const runtimeServerURL = "http://[::1]:50052";

export type ServerType = keyof typeof streamerClients;

const streamerClients = {
  editor: new StreamerClientImpl(
    new StreamingGrpcWebImpl(editorServerURL, {
      debug: false,
    })
  ),
  runtime: new StreamerClientImpl(
    new StreamingGrpcWebImpl(runtimeServerURL, {
      debug: false,
    })
  ),
};

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
  const client = streamerClients[serverType];

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
