/* eslint-disable camelcase */
import type { PropertyInspector } from "@lgn/apis/editor";
import type { NonEmptyArray } from "@lgn/web-client/src/lib/array";
import { filterMap } from "@lgn/web-client/src/lib/array";

import type { Entries } from "@/lib/hierarchyTree";

type SafeRawResourceProperty = Omit<
  PropertyInspector.ResourceProperty,
  "attributes" | "sub_properties"
> & {
  attributes: Record<string, string | undefined>;
  sub_properties: SafeRawResourceProperty[];
};

/** Matches any `ptype` of format "Vec<subPType>" */
const vecPTypeRegExp = /^Vec<(.+)>$/;

/** Matches any `ptype` of format "Option<subPType>" */
const optionPTypeRegExp = /^Option<(.+)>$/;

/** Shared by all resource properties, be it a primitive, a vector, an option, or a component */
type ResourcePropertyBase<Type extends string = string> = {
  ptype: Type;
  name: string;
  attributes: Record<string, string | undefined>;
  sub_properties: ResourceProperty[];
};

export type GroupResourceProperty = ResourcePropertyBase<"group">;

/**
 * Base type used for resource properties that have a `value` field.
 * Extends `ResourcePropertyBase`
 */
type ResourcePropertyWithValueBase<
  Type extends string = string,
  Value = unknown
> = ResourcePropertyBase<Type> & {
  value: Value;
};

export type BooleanProperty = ResourcePropertyWithValueBase<"bool", boolean>;

export type Speed = number;

export type SpeedProperty = ResourcePropertyWithValueBase<"Speed", Speed>;

export type Color = number;

export type ColorProperty = ResourcePropertyWithValueBase<"Color", Color>;

export type StringProperty = ResourcePropertyWithValueBase<"String", string>;

export type NumberProperty = ResourcePropertyWithValueBase<
  "i32" | "u32" | "f32" | "f64" | "usize" | "u8",
  number
>;

export type Vec3 = [number, number, number];

export type Vec3Property = ResourcePropertyWithValueBase<"Vec3", Vec3>;

export type Quat = [number, number, number, number];

export type QuatProperty = ResourcePropertyWithValueBase<"Quat", Quat>;

export type ResourcePathId = string;

export type ResourcePathIdProperty = ResourcePropertyWithValueBase<
  "ResourcePathId",
  ResourcePathId
>;

export type EnumProperty = ResourcePropertyWithValueBase<
  `_enum_:${string}`,
  string
>;

/** List all the possible primitive resources */
export type PrimitiveResourceProperty =
  | BooleanProperty
  | SpeedProperty
  | ColorProperty
  | StringProperty
  | ResourcePathIdProperty
  | EnumProperty
  | NumberProperty
  | Vec3Property
  | QuatProperty;

export type ResourcePropertyWithValue = PrimitiveResourceProperty;

/** Generic resource property type build used for vectors */
export type VecResourceProperty<
  SubProperty extends
    | ResourcePropertyBase<string>
    | ResourcePropertyWithValueBase<string, unknown> =
    | ResourcePropertyBase<string>
    | ResourcePropertyWithValueBase<string, unknown>
> = ResourcePropertyBase<`Vec<${SubProperty["ptype"]}>`>;

/** Generic resource property type build used for options */
export type OptionResourceProperty<
  SubProperty extends
    | ResourcePropertyBase<string>
    | ResourcePropertyWithValueBase<string, unknown> =
    | ResourcePropertyBase<string>
    | ResourcePropertyWithValueBase<string, unknown>
> = ResourcePropertyBase<`Option<${SubProperty["ptype"]}>`>;

/**
 * A Component can have any name, and is defined not by
 * what it _is_ but rather by what it is _not_.
 *
 * A Component is not a Primitive, not an Option, and not a Vec.
 *
 * You can use refinement functions like `propertyIsComponent`
 * to check if a property is a component.
 */
export type ComponentResourceProperty =
  | ResourcePropertyBase<string>
  | ResourcePropertyWithValueBase<string, unknown>;

/**
 * A bag resource property is a property or a group that contains
 * 0 to n properties. They usually don't have the `value` property.
 *
 * A bag is like a Node in a binary tree.
 */
export type BagResourceProperty =
  | GroupResourceProperty
  | OptionResourceProperty<ComponentResourceProperty>
  | VecResourceProperty
  | ComponentResourceProperty;

/**
 * Property unit, typically primitives or optional property
 * that contains a primitive.
 *
 * A unit is like a Leaf in a binary tree.
 */
export type UnitResourceProperty =
  | PrimitiveResourceProperty
  | OptionResourceProperty<PrimitiveResourceProperty>;

/** All the resource property types in an union */
export type ResourceProperty = BagResourceProperty | UnitResourceProperty;

export function propertyIsBoolean(
  property: ResourceProperty
): property is BooleanProperty {
  return property.ptype === "bool";
}

export function propertyIsSpeed(
  property: ResourceProperty
): property is SpeedProperty {
  return property.ptype === "Speed";
}

export function propertyIsColor(
  property: ResourceProperty
): property is ColorProperty {
  return property.ptype === "Color";
}

/**
 * Will return `true` (and implicitly cast) the provided property as a `StringProperty`.
 * There is no such thing as a `ScriptProperty`, a script property is basically a `StringProperty`
 * that contains an `editor_type` attribute.
 */
export function propertyIsScript(
  property: ResourceProperty
): property is StringProperty {
  return property.ptype === "String" && !!property.attributes.editor_type;
}

export function propertyIsString(
  property: ResourceProperty
): property is StringProperty {
  return property.ptype === "String";
}

export function propertyIsResourcePathId(
  property: ResourceProperty
): property is ResourcePathIdProperty {
  return property.ptype === "ResourcePathId";
}

export function propertyIsEnum(
  property: ResourceProperty
): property is EnumProperty {
  return property.ptype.startsWith("_enum_:");
}

export function propertyIsNumber(
  property: ResourceProperty
): property is NumberProperty {
  return ["i32", "u32", "f32", "f64", "u8", "usize"].includes(property.ptype);
}

export function propertyIsVec3(
  property: ResourceProperty
): property is Vec3Property {
  return property.ptype === "Vec3";
}

export function propertyIsQuat(
  property: ResourceProperty
): property is QuatProperty {
  return property.ptype === "Quat";
}

export function propertyIsPrimitive(
  property: ResourceProperty
): property is PrimitiveResourceProperty {
  return [
    propertyIsBoolean,
    propertyIsSpeed,
    propertyIsColor,
    propertyIsString,
    propertyIsResourcePathId,
    propertyIsEnum,
    propertyIsNumber,
    propertyIsVec3,
    propertyIsQuat,
  ].some((predicate) => predicate(property));
}

export function propertyIsVec(
  property: ResourceProperty
): property is VecResourceProperty {
  return vecPTypeRegExp.test(property.ptype);
}

export function propertyIsOption(
  property: ResourceProperty
): property is OptionResourceProperty {
  return optionPTypeRegExp.test(property.ptype);
}

export function propertyIsComponent(
  property: ResourceProperty
): property is ComponentResourceProperty {
  // Using `every` instead of `some` so that it can early return
  // if one of the predicates return `true`
  return ![
    propertyIsPrimitive,
    propertyIsVec,
    propertyIsOption,
    propertyIsGroup,
  ].some((predicate) => predicate(property));
}

export function propertiesAreEntities(property: ResourceWithProperties[]) {
  return property.every(
    (p) => p.description.type.localeCompare("entity") === 0
  );
}

export function propertyIsDynComponent(
  property: ResourceProperty
): property is ComponentResourceProperty {
  return property.ptype.indexOf("<dyn Component>") !== -1;
}

export function propertyIsGroup(
  property: ResourceProperty
): property is GroupResourceProperty {
  return property.ptype === "group";
}

export function propertyIsBag(
  property: ResourceProperty
): property is BagResourceProperty {
  if (
    propertyIsGroup(property) ||
    propertyIsVec(property) ||
    propertyIsComponent(property)
  ) {
    return true;
  }

  if (propertyIsOption(property)) {
    const innerPType = extractOptionPType(property as OptionResourceProperty);

    if (!innerPType) {
      return false;
    }

    return !ptypeBelongsToPrimitive(innerPType);
  }

  return false;
}

/**
 * Extract the inner `ptype` of options:
 *
 * ```typescript
 * extractOptionPType("Option<X>"); // returns "X"
 * extractOptionPType("Nope<Y>"); // return null
 * ```
 */
export function extractOptionPType<
  Property extends PrimitiveResourceProperty | ComponentResourceProperty
>(property: OptionResourceProperty<Property>): Property["ptype"] | null {
  const ptype =
    (property.ptype.match(optionPTypeRegExp)?.[1] as
      | Property["ptype"]
      | undefined) ?? null;

  return ptype;
}

/**
 * Extract the inner `ptype` of arrays/vectors:
 *
 * ```typescript
 * extractVecPType("Vec<X>"); // returns "X"
 * extractVecPType("Nope<Y>"); // return null
 * ```
 */
export function extractVecPType<
  Property extends PrimitiveResourceProperty | ComponentResourceProperty
>(property: VecResourceProperty<Property>): Property["ptype"] | null {
  const ptype =
    (property.ptype.match(vecPTypeRegExp)?.[1] as
      | Property["ptype"]
      | undefined) ?? null;

  return ptype;
}

const primitivePTypes: PrimitiveResourceProperty["ptype"][] = [
  "bool",
  "Speed",
  "Color",
  "String",
  "ResourcePathId",
  "i32",
  "u32",
  "f32",
  "f64",
  "usize",
  "u8",
  "Vec3",
  "Quat",
];

/**
 * Used to work with `ptype`s directly, returns `true` if the `ptype` is known
 * for belonging to a primitive property
 */
export function ptypeBelongsToPrimitive(
  ptype: string
): ptype is PrimitiveResourceProperty["ptype"] {
  return (
    (primitivePTypes as string[]).includes(ptype) || ptype.startsWith("_enum_:")
  );
}

/** Builds an Option property from a property */
export function buildOptionProperty<
  SubProperty extends ResourcePropertyBase | ResourcePropertyWithValueBase
>(name: string, subProperty: SubProperty): OptionResourceProperty<SubProperty> {
  return {
    attributes: {},
    name,
    ptype: `Option<${subProperty.ptype}>`,
    sub_properties: [subProperty],
  };
}

/** Builds an Option property with a `None` value */
export function buildOptionNoneProperty<
  SubProperty extends PrimitiveResourceProperty
>(
  name: string,
  ptype: SubProperty["ptype"]
): OptionResourceProperty<SubProperty> {
  return {
    attributes: {},
    name,
    ptype: `Option<${ptype}>`,
    sub_properties: [],
  };
}

/** Builds a Vec property from a non empty array of properties */
export function buildGroupProperty<SubProperty extends ResourceProperty>(
  name: string,
  sub_properties: SubProperty[]
): GroupResourceProperty {
  return {
    attributes: {},
    name,
    ptype: "group",
    sub_properties,
  };
}

/** Builds a group property from a non empty array of propert */
export function buildVecProperty<
  SubProperty extends ResourcePropertyBase | ResourcePropertyWithValueBase
>(
  name: string,
  sub_properties: NonEmptyArray<SubProperty>
): VecResourceProperty<SubProperty> {
  return {
    attributes: {},
    name,
    ptype: `Vec<${sub_properties[0].ptype}>`,
    sub_properties,
  };
}

// TODO: Drop this when the server can return default values
/** Builds a primitive property from a `ptype` */
export function buildDefaultPrimitiveProperty(
  name: string,
  ptype: PrimitiveResourceProperty["ptype"]
): PrimitiveResourceProperty {
  if (ptype.startsWith("_enum_:")) {
    return {
      ptype,
      name,
      attributes: {},
      sub_properties: [],
      value: "",
    } as EnumProperty;
  }

  switch (ptype) {
    case "Color": {
      return {
        ptype: "Color",
        name,
        attributes: {},
        sub_properties: [],
        value: 0,
      };
    }

    case "Quat": {
      return {
        ptype: "Quat",
        name,
        attributes: {},
        sub_properties: [],
        value: [0, 0, 0, 0],
      };
    }

    case "Speed": {
      return {
        ptype: "Speed",
        name,
        attributes: {},
        sub_properties: [],
        value: 0,
      };
    }

    case "String": {
      return {
        ptype: "String",
        name,
        attributes: {},
        sub_properties: [],
        value: "",
      };
    }

    case "ResourcePathId": {
      return {
        ptype: "ResourcePathId",
        name,
        attributes: {},
        sub_properties: [],
        value: "",
      };
    }

    case "Vec3": {
      return {
        ptype: "Vec3",
        name,
        attributes: {},
        sub_properties: [],
        value: [0, 0, 0],
      };
    }

    case "bool": {
      return {
        ptype: "bool",
        name,
        attributes: {},
        sub_properties: [],
        value: false,
      };
    }

    case "f32":
    case "f64":
    case "i32":
    case "u32":
    case "u8":

    // eslint-disable-next-line no-fallthrough
    case "usize": {
      return {
        ptype,
        name,
        attributes: {},
        sub_properties: [],
        value: 0,
      };
    }
  }

  throw new Error(`Unknown primitive property ptype ${ptype}`);
}

export type ResourceWithProperties = {
  id: string;
  description: PropertyInspector.ResourceDescription;
  version: number;
  properties: ResourceProperty[];
};

function formatOptionProperty(
  property: SafeRawResourceProperty
): OptionResourceProperty | null {
  return {
    name: property.name,
    ptype: property.ptype as OptionResourceProperty["ptype"],
    attributes: property.attributes,
    sub_properties: formatProperties(property.sub_properties),
  };
}

function formatVecProperty(
  property: SafeRawResourceProperty
): VecResourceProperty | null {
  return {
    name: property.name,
    ptype: property.ptype as VecResourceProperty["ptype"],
    attributes: property.attributes,
    sub_properties: formatProperties(property.sub_properties),
  };
}

function formatGroupProperty(
  property: SafeRawResourceProperty
): GroupResourceProperty | ComponentResourceProperty | null {
  return {
    ptype: property.ptype === "_group_" ? "group" : property.ptype,
    name: property.name,
    attributes: property.attributes,
    sub_properties: formatProperties(property.sub_properties),
  };
}

function formatProperty(
  property: SafeRawResourceProperty
): PrimitiveResourceProperty | null {
  if (property.json_value === undefined) {
    return null;
  }

  return {
    name: property.name,
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    value: JSON.parse(property.json_value),
    ptype: property.ptype as PrimitiveResourceProperty["ptype"],
    attributes: property.attributes,
    sub_properties: formatProperties(property.sub_properties),
  };
}

// TODO: Ideally we should get rid of this one
export function formatProperties(
  properties: SafeRawResourceProperty[]
): ResourceProperty[] {
  return filterMap(properties, (property): ResourceProperty | null => {
    if (property.json_value === undefined) {
      if (property.ptype.startsWith("Option")) {
        return formatOptionProperty(property);
      }

      if (property.ptype.startsWith("Vec")) {
        return formatVecProperty(property);
      }

      // We assume unknown properties without a json value are groups
      // TODO: Change this behavior and get rid of the group/virtual-group system
      return formatGroupProperty(property);
    }

    return formatProperty(property);
  });
}

/** Retrieves the resource name from the resource `Entries` based on the provided value string */
export function getResourceNameFromEntries(
  resourceEntries: Entries<PropertyInspector.ResourceDescription>,
  value: string
): string {
  const entry = resourceEntries.find((entry) =>
    value.startsWith(entry.item.id)
  );

  let result = "";

  if (entry) {
    result = entry.name;

    let index = value.indexOf("_");

    if (index !== -1) {
      const subValue = value.slice(index + 1);

      index = subValue.indexOf("|");

      if (index !== -1) {
        result += "/" + subValue.slice(undefined, index);
      }
    }
  }

  return result;
}

export function getResourceType(
  property: ResourceProperty,
  parentProperty: BagResourceProperty | null
): string | null {
  let resourceType = property.attributes.resource_type;

  if (
    resourceType === undefined &&
    parentProperty &&
    (propertyIsVec(parentProperty) || propertyIsOption(parentProperty))
  ) {
    resourceType = parentProperty.attributes.resource_type;
  }

  if (resourceType) {
    const index = resourceType.lastIndexOf(":");

    if (index !== -1) {
      resourceType = resourceType.slice(index + 1);
    }
  }

  return typeof resourceType === "string" ? resourceType : null;
}

export function isPropertyDisplayable(propertyName: string, value: string) {
  return propertyName.toLocaleLowerCase().includes(value.toLocaleLowerCase());
}
