import {
  GrpcWebImpl as EditorGrpcWebImpl,
  EditorClientImpl,
  ResourceDescription,
} from "@lgn/proto-editor/codegen/editor";
import {
  GrpcWebImpl as StreamingGrpcWebImpl,
  StreamerClientImpl,
} from "@lgn/proto-streaming/codegen/streaming";

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

export type ResourceProperty = ResourceWithProperties["properties"][number];

/**
 * Fetch a resource's properties using its ID
 * @param resource The resource description with the ID and the version
 * @returns The properties of the resource and possibly its description
 */
export async function getResourceProperties({
  id,
  version,
}: ResourceDescription) {
  const { description, properties } = await editorClient.getResourceProperties({
    id,
  });

  if (!description) {
    throw new Error("Fetched resource didn't return any description");
  }

  return {
    id,
    description,
    version,
    properties: properties.map((property) => {
      const value = JSON.parse(new TextDecoder().decode(property.value));
      const defaultValue = JSON.parse(
        new TextDecoder().decode(property.defaultValue)
      );

      return {
        ...property,
        defaultValue:
          property.ptype === "color"
            ? defaultValue.toString(16).padStart(8, "0")
            : defaultValue,
        value:
          property.ptype === "color"
            ? value.toString(16).padStart(8, "0")
            : value,
      };
    }),
  };
}

type PropertyUpdate = {
  name: string;
  // Can be any JSON serializable value
  value: unknown;
};

/**
 * Update a resource's properties
 * @param resourceId The resource ID
 * @param version
 * @param propertyUpdates
 * @returns
 */
export async function updateResourceProperties(
  resourceId: string,
  version: number,
  propertyUpdates: PropertyUpdate[]
) {
  await editorClient.updateResourceProperties({
    id: resourceId,
    version,
    propertyUpdates: propertyUpdates.map((propertyUpdate) => ({
      ...propertyUpdate,
      value: new TextEncoder().encode(JSON.stringify(propertyUpdate.value)),
    })),
  });
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
