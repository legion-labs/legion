import log from "@lgn/frontend/src/lib/log";
import {
  GrpcWebImpl as EditorResourceBrowserWebImpl,
  ResourceBrowserClientImpl,
  ResourceDescription,
} from "@lgn/proto-editor/codegen/resource_browser";
import {
  GrpcWebImpl as EditorPropertyInspectorWebImpl,
  PropertyInspectorClientImpl,
  ResourceProperty as ResourceRawProperty,
} from "@lgn/proto-editor/codegen/property_inspector";

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
export async function getAllResources() {
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

  return getMoreResources("");
}

type ResourcePropertyCommon<Type extends string = string> = {
  ptype: Type;
  name: string;
  attributes: Record<string, string>;
  subProperties: (ResourceProperty | ResourcePropertyGroup)[];
};

export type ResourcePropertyGroup = ResourcePropertyCommon<
  "virtual-group" | "group"
>;

type ResourcePropertyBase<
  Type extends string,
  Value
> = ResourcePropertyCommon<Type> & {
  value: Value;
};

export type BooleanProperty = ResourcePropertyBase<"bool", boolean>;

export type Speed = number;

export type SpeedProperty = ResourcePropertyBase<"speed", Speed>;

export type Color = number;

export type ColorProperty = ResourcePropertyBase<"color", Color>;

export type StringProperty = ResourcePropertyBase<"string", string>;

export type NumberProperty = ResourcePropertyBase<
  "i32" | "u32" | "f32" | "f64" | "usize" | "u8",
  number
>;

export type Vec3 = [number, number, number];

export type Vec3Property = ResourcePropertyBase<"vec3", Vec3>;

export type Quat = [number, number, number, number];

export type QuatProperty = ResourcePropertyBase<"quat", Quat>;

// Uint8Array might fit better here, but it requires some value conversion at runtime
export type VecU8 = number[];

export type VecU8Property = ResourcePropertyBase<"vec<u8>", VecU8>;

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
  property: ResourceProperty | ResourcePropertyGroup
): property is BooleanProperty {
  return property.ptype.toLowerCase() === "bool";
}

export function propertyIsSpeed(
  property: ResourceProperty | ResourcePropertyGroup
): property is SpeedProperty {
  return property.ptype.toLowerCase() === "speed";
}

export function propertyIsColor(
  property: ResourceProperty | ResourcePropertyGroup
): property is ColorProperty {
  return property.ptype.toLowerCase() === "color";
}

export function propertyIsString(
  property: ResourceProperty | ResourcePropertyGroup
): property is StringProperty {
  return property.ptype.toLowerCase() === "string";
}

export function propertyIsNumber(
  property: ResourceProperty | ResourcePropertyGroup
): property is NumberProperty {
  return ["i32", "u32", "f32", "f64", "u8", "usize"].includes(
    property.ptype.toLowerCase()
  );
}

export function propertyIsVec3(
  property: ResourceProperty | ResourcePropertyGroup
): property is Vec3Property {
  return property.ptype.toLowerCase() === "vec3";
}

export function propertyIsQuat(
  property: ResourceProperty | ResourcePropertyGroup
): property is QuatProperty {
  return property.ptype.toLowerCase() === "quat";
}

export function propertyIsVecU8(
  property: ResourceProperty | ResourcePropertyGroup
): property is VecU8Property {
  return property.ptype.toLowerCase() === "vec<u8>";
}

export function propertyIsGroup(
  property: ResourceProperty | ResourcePropertyGroup
): property is ResourcePropertyGroup {
  return ["virtual-group", "group"].includes(property.ptype.toLowerCase());
}

export function propertyIsVirtualGroup(
  property: ResourceProperty | ResourcePropertyGroup
): property is ResourcePropertyGroup {
  return property.ptype.toLowerCase() === "virtual-group";
}

export type ResourceWithProperties = {
  id: string;
  description: ResourceDescription;
  version: number;
  properties: (ResourceProperty | ResourcePropertyGroup)[];
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
  const { description, properties } =
    await propertyInspectorClient.getResourceProperties({
      id,
    });

  if (!description) {
    throw new Error("Fetched resource didn't return any description");
  }

  function formatProperties(
    properties: ResourceRawProperty[]
  ): (ResourceProperty | ResourcePropertyGroup)[] {
    return properties.map(
      (property): ResourceProperty | ResourcePropertyGroup => {
        if (!property.jsonValue) {
          return {
            ptype: property.ptype === "_group_" ? "virtual-group" : "group",
            name: property.name,
            attributes: property.attributes,
            subProperties: formatProperties(property.subProperties),
          };
        }

        return {
          name: property.name,
          value: JSON.parse(property.jsonValue),
          // We don't actually validate the incoming data to keep it fast
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          ptype: property.ptype as ResourceProperty["ptype"],
          attributes: property.attributes,
          subProperties: formatProperties(property.subProperties),
        };
      }
    );
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
 * Used for logging purpose
 * @param jsonCommand
 * @returns
 */
export async function onSendEditionCommand(jsonCommand: string) {
  log.info("video", `Sending edition_command=${jsonCommand}`);
}
