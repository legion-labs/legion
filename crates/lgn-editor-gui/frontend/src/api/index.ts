import type { Observable } from "rxjs";
import log from "@lgn/web-client/src/lib/log";
import {
  GrpcWebImpl as EditorResourceBrowserWebImpl,
  ResourceBrowserClientImpl,
  ResourceDescription,
} from "@lgn/proto-editor/dist/resource_browser";
import {
  GrpcWebImpl as EditorImpl,
  EditorClientImpl,
} from "@lgn/proto-editor/dist/editor";
import {
  GrpcWebImpl as EditorPropertyInspectorWebImpl,
  PropertyInspectorClientImpl,
} from "@lgn/proto-editor/dist/property_inspector";
import {
  GrpcWebImpl as EditorSourceControlWebImpl,
  SourceControlClientImpl,
  UploadRawFileResponse,
} from "@lgn/proto-editor/dist/source_control";
import { formatProperties } from "../lib/propertyGrid";
import type {
  ResourcePropertyWithValue,
  ResourceWithProperties,
} from "../lib/propertyGrid";

const defaultEditorServerURL = "http://[::1]:50051";

let resourceBrowserClient: ResourceBrowserClientImpl;

let propertyInspectorClient: PropertyInspectorClientImpl;

let sourceControlClient: SourceControlClientImpl;

let editorClient: EditorClientImpl;

export function initApiClient({
  editorServerUrl = defaultEditorServerURL,
}: {
  editorServerUrl?: string;
}) {
  resourceBrowserClient = new ResourceBrowserClientImpl(
    new EditorResourceBrowserWebImpl(editorServerUrl, { debug: false })
  );

  propertyInspectorClient = new PropertyInspectorClientImpl(
    new EditorPropertyInspectorWebImpl(editorServerUrl, { debug: false })
  );

  sourceControlClient = new SourceControlClientImpl(
    new EditorSourceControlWebImpl(editorServerUrl, {
      debug: false,
    })
  );

  editorClient = new EditorClientImpl(
    new EditorImpl(editorServerUrl, {
      debug: false,
    })
  );
}

/**
 * Eagerly fetches all the resource descriptions on the server
 * @returns All the resource descriptions
 */
export async function getAllResources(searchToken = "") {
  const resourceDescriptions: ResourceDescription[] = [];

  async function getMoreResources(
    searchToken: string
  ): Promise<ResourceDescription[]> {
    const response = await resourceBrowserClient.searchResources({
      searchToken,
    });

    resourceDescriptions.push(...response.resourceDescriptions);

    return response.nextSearchToken
      ? getMoreResources(response.nextSearchToken)
      : resourceDescriptions;
  }

  const allResources = await getMoreResources(searchToken);

  return allResources.sort((resource1, resource2) =>
    resource1.path > resource2.path ? 1 : -1
  );
}

/**
 * Fetch a resource's properties using its ID
 * @param resource The resource description with the ID and the version
 * @returns The properties of the resource and possibly its description
 */
export async function getResourceProperties(
  id: string
): Promise<ResourceWithProperties> {
  const { description, properties } =
    await propertyInspectorClient.getResourceProperties({
      id,
    });

  if (!description) {
    throw new Error("Fetched resource didn't return any description");
  }

  return {
    id,
    description,
    version: description.version,
    properties: formatProperties(properties),
  };
}

export type PropertyUpdate = {
  name: string;
  // Can be any JSON serializable value
  value: ResourcePropertyWithValue["value"] | null;
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
  await propertyInspectorClient.updateResourceProperties({
    id: resourceId,
    version,
    propertyUpdates: propertyUpdates.map(({ name, value }) => ({
      name: name,
      jsonValue: JSON.stringify(value),
    })),
  });
}

/**
 * Update selection
 * @param resourceId The resource ID
 * @returns
 */
export async function updateSelection(resourceId: string) {
  await propertyInspectorClient.updateSelection({
    resourceId: resourceId,
  });
}

export type AddVectorSubProperty = {
  path: string;
  index: number;
  jsonValue: string | undefined;
};

export async function addPropertyInPropertyVector(
  resourceId: string,
  { path, index, jsonValue }: AddVectorSubProperty
) {
  const result = await propertyInspectorClient.insertNewArrayElement({
    resourceId,
    arrayPath: path,
    index,
    jsonValue,
  });

  const value = result.newValue;

  if (value) {
    window.dispatchEvent(
      new CustomEvent("refresh-property", {
        detail: { path, value },
      })
    );
  }
}

export type RemoveVectorSubProperty = {
  path: string;
  indices: number[];
};

export async function removeVectorSubProperty(
  resourceId: string,
  { path, indices }: RemoveVectorSubProperty
) {
  await propertyInspectorClient.deleteArrayElement({
    resourceId,
    arrayPath: path,
    indices,
  });
}

export async function getResourceTypes() {
  return resourceBrowserClient.getResourceTypeNames({});
}

export async function getAvailableComponentTypes() {
  return propertyInspectorClient.getAvailableDynTraits({
    traitName: "dyn Component",
  });
}

export async function createResource({
  resourceName,
  resourceType,
  parentResourceId,
}: {
  resourceName: string;
  resourceType: string;
  parentResourceId: string | undefined;
}) {
  return resourceBrowserClient.createResource({
    resourceName,
    resourceType,
    parentResourceId,
  });
}

export async function renameResource({
  id,
  newPath,
}: {
  id: string;
  newPath: string;
}) {
  return resourceBrowserClient.renameResource({
    id,
    newPath,
  });
}

export async function removeResource({ id }: { id: string }) {
  return resourceBrowserClient.deleteResource({ id });
}

export async function cloneResource({
  sourceId,
  targetParentId,
}: {
  sourceId: string;
  targetParentId?: string;
}) {
  return resourceBrowserClient.cloneResource({ sourceId, targetParentId });
}

/**
 * Used for logging purpose
 * @param jsonCommand
 * @returns
 */
export async function onSendEditionCommand(jsonCommand: string) {
  log.info("video", `Sending edition_command=${jsonCommand}`);
}

export async function initFileUpload({
  name,
  size,
}: {
  name: string;
  size: number;
}) {
  return sourceControlClient.initUploadRawFile({ name, size });
}

export function streamFileUpload({
  id,
  content,
}: {
  id: string;
  content: Uint8Array;
}): Observable<UploadRawFileResponse> {
  return sourceControlClient.uploadRawFile({
    id,
    content,
  });
}

// FIXME: This function is known for being broken
// the api is not fully over yet and it might change soon
export function reparentResources({
  id,
  newPath,
}: {
  id: string;
  newPath: string;
}) {
  return resourceBrowserClient.reparentResource({
    id,
    newPath,
  });
}

export function syncLatest() {
  return sourceControlClient.syncLatest({});
}

export function commitStagedResources({ message }: { message: string }) {
  return sourceControlClient.commitStagedResources({
    message,
  });
}

export function getStagedResources() {
  return sourceControlClient.getStagedResources({});
}

export async function openScene({ id }: { id: string }) {
  return resourceBrowserClient.openScene({ id });
}

export async function closeScene({ id }: { id: string }) {
  return resourceBrowserClient.closeScene({ id });
}

export async function initLogStream() {
  return editorClient.initLogStream({});
}

export async function initMessageStream() {
  return editorClient.initMessageStream({});
}
