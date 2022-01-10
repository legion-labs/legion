import { ResourceDescription } from "@lgn/proto-editor/codegen/resource_browser";
import { ResourceProperty as RawResourceProperty } from "@lgn/proto-editor/codegen/property_inspector";
import log from "@lgn/frontend/src/lib/log";
import { filterMap } from "../lib/array";

/** Matches any `ptype` of format "Vec<subPType>" */
const vecPTypeRegExp = /^Vec\<(.*)\>$/;

/** Matches any `ptype` of format "Option<subPType>" */
const optionPTypeRegExp = /^Option\<(.*)\>$/;

/** Matches any `ptype` of format "Component<subPType>" */
const componentPTypeRegExp = /^Component\<(.*)\>$/;

/** Shared by all resource properties, be it a primitive, a vector, an option, or a component */
type ResourcePropertyBase<Type extends string = string> = {
  ptype: Type;
  name: string;
  attributes: Record<string, string>;
  subProperties: ResourceProperty[];
};

export type GroupResourceProperty = ResourcePropertyBase<"group">;

export type ComponentResourceProperty<ComponentName extends string = string> =
  ResourcePropertyBase<`Component<${ComponentName}>`>;

/**
 * Base type used for resource properties that have a `value` field.
 * Extends `ResourcePropertyBase`
 */
type ResourcePropertyWithValueBase<
  Type extends string,
  Value
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

/** List all the possible primitive resources */
export type PrimitiveResourceProperty =
  | BooleanProperty
  | SpeedProperty
  | ColorProperty
  | StringProperty
  | NumberProperty
  | Vec3Property
  | QuatProperty;

/** Generic resource property type build used for vectors */
export type VecResourceProperty<
  Property extends PrimitiveResourceProperty = PrimitiveResourceProperty
> = ResourcePropertyBase<`Vec<${Property["ptype"]}>`>;

/** Generic resource property type build used for options */
export type OptionResourceProperty<
  Property extends PrimitiveResourceProperty = PrimitiveResourceProperty
> = ResourcePropertyBase<`Option<${Property["ptype"]}>`>;

export type ResourcePropertyWithValue = PrimitiveResourceProperty;

export type ResourcePropertyNoValue =
  | OptionResourceProperty
  | VecResourceProperty
  | GroupResourceProperty
  | ComponentResourceProperty;

export type ResourceProperty =
  | ResourcePropertyWithValue
  | ResourcePropertyNoValue;

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

export function propertyIsString(
  property: ResourceProperty
): property is StringProperty {
  return property.ptype === "String";
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

export function propertyIsComponent(
  property: ResourceProperty
): property is ComponentResourceProperty {
  return componentPTypeRegExp.test(property.ptype);
}

export function propertyIsGroup(
  property: ResourceProperty
): property is GroupResourceProperty {
  return ["virtual-group", "group"].includes(property.ptype);
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

export type ResourceWithProperties = {
  id: string;
  description: ResourceDescription;
  version: number;
  properties: ResourceProperty[];
};

function formatOptionProperty(
  property: RawResourceProperty
): OptionResourceProperty | null {
  const ptype = property.ptype.match(optionPTypeRegExp)?.[1].trim();

  if (!ptype) {
    log.debug(`Resource "ptype" seems to be invalid: ${property.ptype}`);

    return null;
  }

  return {
    name: property.name,
    ptype: property.ptype as OptionResourceProperty["ptype"],
    attributes: property.attributes,
    subProperties: formatProperties(property.subProperties),
  };
}

function formatVecProperty(
  property: RawResourceProperty
): VecResourceProperty | null {
  const ptype = property.ptype.match(vecPTypeRegExp)?.[1].trim();

  if (!ptype) {
    log.debug(`Resource "ptype" seems to be invalid: ${property.ptype}`);

    return null;
  }

  return {
    name: property.name,
    ptype: property.ptype as VecResourceProperty["ptype"],
    attributes: property.attributes,
    subProperties: formatProperties(property.subProperties),
  };
}

function formatGroupProperty(
  property: RawResourceProperty
): GroupResourceProperty | ComponentResourceProperty | null {
  if (property.ptype !== "_group_") {
    return {
      ptype: `Component<${property.ptype}>`,
      name: property.name,
      attributes: property.attributes,
      subProperties: formatProperties(property.subProperties),
    };
  }

  return {
    ptype: "group",
    name: property.name,
    attributes: property.attributes,
    subProperties: formatProperties(property.subProperties),
  };
}

function formatProperty(
  property: RawResourceProperty
): PrimitiveResourceProperty | null {
  if (!property.jsonValue) {
    return null;
  }

  return {
    name: property.name,
    value: JSON.parse(property.jsonValue),
    ptype: property.ptype as PrimitiveResourceProperty["ptype"],
    attributes: property.attributes,
    subProperties: formatProperties(property.subProperties),
  };
}

export function formatProperties(
  properties: RawResourceProperty[]
): ResourceProperty[] {
  return filterMap(properties, (property): ResourceProperty | null => {
    if (!property.jsonValue) {
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
