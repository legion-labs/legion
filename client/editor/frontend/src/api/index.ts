import log from "@lgn/frontend/src/lib/log";
import {
  GrpcWebImpl as EditorGrpcWebImpl,
  EditorClientImpl,
  ResourceDescription,
} from "@lgn/proto-editor/codegen/editor";

const editorServerURL = "http://[::1]:50051";

const editorClient = new EditorClientImpl(
  new EditorGrpcWebImpl(editorServerURL, {
    debug: false,
  })
);

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

type ResourcePropertyBase<Value, Type extends string> = {
  defaultValue: Value;
  value: Value;
  name: string;
  ptype: Type;
  group: string;
};

export type BooleanProperty = ResourcePropertyBase<boolean, "bool">;

export type Speed = number;

export type SpeedProperty = ResourcePropertyBase<Speed, "speed">;

export type Color = number;

export type ColorProperty = ResourcePropertyBase<Color, "color">;

export type StringProperty = ResourcePropertyBase<string, "string">;

export type NumberProperty = ResourcePropertyBase<
  number,
  "i32" | "u32" | "f32" | "f64"
>;

export type Vec3 = [number, number, number];

export type Vec3Property = ResourcePropertyBase<Vec3, "vec3">;

export type Quat = [number, number, number, number];

export type QuatProperty = ResourcePropertyBase<Quat, "quat">;

// Uint8Array might fit better here, but it requires some value conversion at runtime
export type VecU8 = number[];

export type VecU8Property = ResourcePropertyBase<VecU8, "vec < u8 >">;

export type ResourceProperty =
  | BooleanProperty
  | SpeedProperty
  | ColorProperty
  | StringProperty
  | NumberProperty
  | Vec3Property
  | QuatProperty
  | VecU8Property;

export function propertyIsBoolean(
  property: ResourceProperty
): property is BooleanProperty {
  return property.ptype === "bool";
}

export function propertyIsSpeed(
  property: ResourceProperty
): property is SpeedProperty {
  return property.ptype === "speed";
}

export function propertyIsColor(
  property: ResourceProperty
): property is ColorProperty {
  return property.ptype === "color";
}

export function propertyIsString(
  property: ResourceProperty
): property is StringProperty {
  return property.ptype === "string";
}

export function propertyIsNumber(
  property: ResourceProperty
): property is NumberProperty {
  return ["i32", "u32", "f32", "f64", "usize"].includes(property.ptype);
}

export function propertyIsVec3(
  property: ResourceProperty
): property is Vec3Property {
  return property.ptype === "vec3";
}

export function propertyIsQuat(
  property: ResourceProperty
): property is QuatProperty {
  return property.ptype === "quat";
}

export function propertyIsVecU8(
  property: ResourceProperty
): property is VecU8Property {
  return property.ptype === "vec < u8 >";
}

export type ResourceWithProperties = {
  id: string;
  description: ResourceDescription;
  version: number;
  properties: ResourceProperty[];
};

/**
 * Fetch a resource's properties using its ID
 * @param resource The resource description with the ID and the version
 * @returns The properties of the resource and possibly its description
 */
export async function getResourceProperties({
  id,
  version,
}: ResourceDescription): Promise<ResourceWithProperties> {
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
        defaultValue,
        value,
        // We don't actually validate the incoming data to keep it fast
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        ptype: property.ptype as ResourceProperty["ptype"],
      };
    }),
  };
}

export type PropertyUpdate = {
  name: string;
  // Can be any JSON serializable value
  value: ResourceProperty["value"];
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

/**
 * Used for logging purpose
 * @param jsonCommand
 * @returns
 */
export async function onSendEditionCommand(jsonCommand: string) {
  log.info("video", `Sending edition_command=${jsonCommand}`);
}
