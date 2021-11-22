import {
  GrpcWebImpl as EditorGrpcWebImpl,
  EditorClientImpl,
  ResourceDescription,
  GetResourcePropertiesResponse,
} from "@/proto/editor/editor";
import {
  GrpcWebImpl as StreamingGrpcWebImpl,
  StreamerClientImpl,
} from "@/proto/streaming/streaming";

// TODO: Move to config
const serverUrl = "http://[::1]:50051";

// Some functions useful when dealing with the api

const stringToBytes = (s: string) => new TextEncoder().encode(s);

const jsonToBytes = (j: Record<string, unknown>) =>
  stringToBytes(JSON.stringify(j));

const bytesToString = (b: Uint8Array) => new TextDecoder().decode(b);

const bytesToJson = <T>(b: Uint8Array): T => JSON.parse(bytesToString(b));

// Our API (gRPC) clients

export const editorClient = new EditorClientImpl(
  new EditorGrpcWebImpl(serverUrl, {
    // TODO: Should be true in dev mode
    debug: false,
  })
);

export const streamerClient = new StreamerClientImpl(
  new StreamingGrpcWebImpl(serverUrl, {
    // TODO: Should be true in dev mode
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
 * Eagerly fetches all the resource descriptions on the server
 * @returns All the resource descriptions
 */
export async function getAllResources() {
  const resourceDescriptions: ResourceDescription[] = [];

  async function getMoreResources(
    searchToken: string
  ): Promise<ResourceDescription[]> {
    const response = await editorClient.searchResources({
      searchToken,
    });

    resourceDescriptions.push(...response.resourceDescriptions);

    return response.nextSearchToken
      ? getMoreResources(response.nextSearchToken)
      : resourceDescriptions;
  }

  return getMoreResources("");
}

export function getResourceProperties(
  resourceId: string
): Promise<GetResourcePropertiesResponse> {
  return editorClient.getResourceProperties({ id: resourceId });
}

// TODO: Implement logging
/**
 * Used mostly for logging purpose
 * @param _jsonMsg
 * @returns
 */
export async function onReceiveControlMessage(_jsonMsg: string) {
  return;
}

// TODO: Implement logging
/**
 * Used mostly for logging purpose
 * @param _jsonMsg
 * @returns
 */
export async function onSendEditionCommand(_jsonCommand: string) {
  return;
}

// TODO: Implement logging
/**
 * Used mostly for logging purpose
 * @param _chunkHeader
 * @returns
 */
export async function onVideoChunkReceived(_chunkHeader: string) {
  return;
}

// TODO: Implement logging
/**
 * Used mostly for logging purpose
 * @returns
 */
export async function onVideoClose() {
  return;
}
