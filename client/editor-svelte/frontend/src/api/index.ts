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

// TODO: Improve type safety for properties
export type ResourceWithProperties = Awaited<
  ReturnType<typeof getResourceProperties>
>;

/**
 * Fetch a resource's properties from an ID
 * @param resourceId The resource ID
 * @returns The properties of the resource and possibly its description
 */
export async function getResourceProperties(resourceId: string) {
  const { description, properties } = await editorClient.getResourceProperties({
    id: resourceId,
  });

  if (!description) {
    throw new Error("Fetched resource didn't return any description");
  }

  properties
    .filter((p) => p.ptype === "color")
    .forEach((p) =>
      console.log(
        `COLOR: ${p.value}/${new TextDecoder().decode(p.value)} - ${
          p.defaultValue
        }/${new TextDecoder().decode(p.defaultValue)}`
      )
    );

  return {
    description,
    properties: properties.map((property) => {
      const value = JSON.parse(new TextDecoder().decode(property.value));
      const defaultValue = JSON.parse(
        new TextDecoder().decode(property.defaultValue)
      );

      return {
        ...property,
        defaultValue:
          // TODO: Support color alpha
          property.ptype === "color"
            ? `#${defaultValue.toString(16).padStart(8, "0").slice(0, 6)}`
            : defaultValue,
        value:
          property.ptype === "color"
            ? `#${value.toString(16).padStart(8, "0").slice(0, 6)}`
            : value,
      };
    }),
  };
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
