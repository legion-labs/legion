import log from "@/lib/log";
import {} from "@lgn/proto-runtime/codegen/runtime";
import {
  GrpcWebImpl as StreamingGrpcWebImpl,
  StreamerClientImpl,
} from "@lgn/proto-streaming/codegen/streaming";

// TODO: Move to config
const serverUrl = "http://[::1]:50052";

// Some functions useful when dealing with the api

const stringToBytes = (s: string) => new TextEncoder().encode(s);

const jsonToBytes = (j: Record<string, unknown>) =>
  stringToBytes(JSON.stringify(j));

const bytesToString = (b: Uint8Array) => new TextDecoder().decode(b);

const bytesToJson = <T>(b: Uint8Array): T => JSON.parse(bytesToString(b));

export const streamerClient = new StreamerClientImpl(
  new StreamingGrpcWebImpl(serverUrl, {
    debug: false,
  })
);

/**
 * Initialize the video player stream
 * @param localSessionDescription
 * @returns a valid RTC sessions description to use with an RTCPeerConnection
 */
export async function initializeStream(
  localSessionDescription: RTCSessionDescription
) {
  const response = await streamerClient.initializeStream({
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

/**
 * Used for logging purpose
 * @param jsonCommand
 * @returns
 */
export async function onSendEditionCommand(jsonCommand: string) {
  log.info("video", `Sending edition_command=${jsonCommand}`);
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
