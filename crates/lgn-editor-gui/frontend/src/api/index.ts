import log from "@lgn/web-client/src/lib/log";
import {
  GrpcWebImpl as EditorResourceBrowserWebImpl,
  ResourceBrowserClientImpl,
  ResourceDescription,
} from "@lgn/proto-editor/dist/resource_browser";
import {
  GrpcWebImpl as EditorPropertyInspectorWebImpl,
  PropertyInspectorClientImpl,
} from "@lgn/proto-editor/dist/property_inspector";
import {
  formatProperties,
  ResourcePropertyWithValue,
  ResourceWithProperties,
} from "../lib/propertyGrid";

const editorServerURL = "http://[::1]:50051";

const resourceBrowserClient = new ResourceBrowserClientImpl(
  new EditorResourceBrowserWebImpl(editorServerURL, { debug: false })
);

const propertyInspectorClient = new PropertyInspectorClientImpl(
  new EditorPropertyInspectorWebImpl(editorServerURL, { debug: false })
);

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
export async function getResourceProperties({
  id,
  version,
}: ResourceDescription): Promise<ResourceWithProperties> {
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
    version,
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
  let result = await propertyInspectorClient.insertNewArrayElement({
    resourceId,
    arrayPath: path,
    index,
    jsonValue,
  });

  let value = result.newValue;
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
  resourcePath,
  resourceType,
}: {
  resourcePath: string;
  resourceType: string;
}) {
  return resourceBrowserClient.createResource({
    resourcePath,
    resourceType,
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
