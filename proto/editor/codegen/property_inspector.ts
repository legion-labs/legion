/* eslint-disable */
import Long from "long";
import { grpc } from "@improbable-eng/grpc-web";
import _m0 from "protobufjs/minimal";
import { BrowserHeaders } from "browser-headers";

export const protobufPackage = "property_inspector";

export interface GetResourcePropertiesRequest {
  id: string;
}

export interface ResourceDescription {
  id: string;
  path: string;
  version: number;
}

export interface GetResourcePropertiesResponse {
  description: ResourceDescription | undefined;
  properties: ResourceProperty[];
}

export interface ResourceProperty {
  name: string;
  ptype: string;
  jsonValue?: string | undefined;
  subProperties: ResourceProperty[];
  attributes: { [key: string]: string };
}

export interface ResourceProperty_AttributesEntry {
  key: string;
  value: string;
}

export interface UpdateResourcePropertiesRequest {
  id: string;
  version: number;
  propertyUpdates: ResourcePropertyUpdate[];
}

export interface UpdateResourcePropertiesResponse {}

export interface ResourcePropertyUpdate {
  name: string;
  jsonValue: string;
}

export interface DeleteArrayElementRequest {
  resourceId: string;
  arrayPath: string;
  indices: number[];
}

export interface DeleteArrayElementResponse {}

export interface InsertNewArrayElementRequest {
  resourceId: string;
  arrayPath: string;
  index: number;
  jsonValue: string;
}

export interface InsertNewArrayElementResponse {}

export interface ReorderArrayElementRequest {
  resourceId: string;
  arrayPath: string;
  oldIndex: number;
  newIndex: number;
}

export interface ReorderArrayElementResponse {}

const baseGetResourcePropertiesRequest: object = { id: "" };

export const GetResourcePropertiesRequest = {
  encode(
    message: GetResourcePropertiesRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): GetResourcePropertiesRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseGetResourcePropertiesRequest,
    } as GetResourcePropertiesRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): GetResourcePropertiesRequest {
    const message = {
      ...baseGetResourcePropertiesRequest,
    } as GetResourcePropertiesRequest;
    message.id =
      object.id !== undefined && object.id !== null ? String(object.id) : "";
    return message;
  },

  toJSON(message: GetResourcePropertiesRequest): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<GetResourcePropertiesRequest>, I>>(
    object: I
  ): GetResourcePropertiesRequest {
    const message = {
      ...baseGetResourcePropertiesRequest,
    } as GetResourcePropertiesRequest;
    message.id = object.id ?? "";
    return message;
  },
};

const baseResourceDescription: object = { id: "", path: "", version: 0 };

export const ResourceDescription = {
  encode(
    message: ResourceDescription,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.path !== "") {
      writer.uint32(18).string(message.path);
    }
    if (message.version !== 0) {
      writer.uint32(24).uint32(message.version);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ResourceDescription {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseResourceDescription } as ResourceDescription;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.path = reader.string();
          break;
        case 3:
          message.version = reader.uint32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ResourceDescription {
    const message = { ...baseResourceDescription } as ResourceDescription;
    message.id =
      object.id !== undefined && object.id !== null ? String(object.id) : "";
    message.path =
      object.path !== undefined && object.path !== null
        ? String(object.path)
        : "";
    message.version =
      object.version !== undefined && object.version !== null
        ? Number(object.version)
        : 0;
    return message;
  },

  toJSON(message: ResourceDescription): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.path !== undefined && (obj.path = message.path);
    message.version !== undefined &&
      (obj.version = Math.round(message.version));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ResourceDescription>, I>>(
    object: I
  ): ResourceDescription {
    const message = { ...baseResourceDescription } as ResourceDescription;
    message.id = object.id ?? "";
    message.path = object.path ?? "";
    message.version = object.version ?? 0;
    return message;
  },
};

const baseGetResourcePropertiesResponse: object = {};

export const GetResourcePropertiesResponse = {
  encode(
    message: GetResourcePropertiesResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.description !== undefined) {
      ResourceDescription.encode(
        message.description,
        writer.uint32(10).fork()
      ).ldelim();
    }
    for (const v of message.properties) {
      ResourceProperty.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): GetResourcePropertiesResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseGetResourcePropertiesResponse,
    } as GetResourcePropertiesResponse;
    message.properties = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.description = ResourceDescription.decode(
            reader,
            reader.uint32()
          );
          break;
        case 2:
          message.properties.push(
            ResourceProperty.decode(reader, reader.uint32())
          );
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): GetResourcePropertiesResponse {
    const message = {
      ...baseGetResourcePropertiesResponse,
    } as GetResourcePropertiesResponse;
    message.description =
      object.description !== undefined && object.description !== null
        ? ResourceDescription.fromJSON(object.description)
        : undefined;
    message.properties = (object.properties ?? []).map((e: any) =>
      ResourceProperty.fromJSON(e)
    );
    return message;
  },

  toJSON(message: GetResourcePropertiesResponse): unknown {
    const obj: any = {};
    message.description !== undefined &&
      (obj.description = message.description
        ? ResourceDescription.toJSON(message.description)
        : undefined);
    if (message.properties) {
      obj.properties = message.properties.map((e) =>
        e ? ResourceProperty.toJSON(e) : undefined
      );
    } else {
      obj.properties = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<GetResourcePropertiesResponse>, I>>(
    object: I
  ): GetResourcePropertiesResponse {
    const message = {
      ...baseGetResourcePropertiesResponse,
    } as GetResourcePropertiesResponse;
    message.description =
      object.description !== undefined && object.description !== null
        ? ResourceDescription.fromPartial(object.description)
        : undefined;
    message.properties =
      object.properties?.map((e) => ResourceProperty.fromPartial(e)) || [];
    return message;
  },
};

const baseResourceProperty: object = { name: "", ptype: "" };

export const ResourceProperty = {
  encode(
    message: ResourceProperty,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.ptype !== "") {
      writer.uint32(18).string(message.ptype);
    }
    if (message.jsonValue !== undefined) {
      writer.uint32(34).string(message.jsonValue);
    }
    for (const v of message.subProperties) {
      ResourceProperty.encode(v!, writer.uint32(42).fork()).ldelim();
    }
    Object.entries(message.attributes).forEach(([key, value]) => {
      ResourceProperty_AttributesEntry.encode(
        { key: key as any, value },
        writer.uint32(50).fork()
      ).ldelim();
    });
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ResourceProperty {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseResourceProperty } as ResourceProperty;
    message.subProperties = [];
    message.attributes = {};
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.ptype = reader.string();
          break;
        case 4:
          message.jsonValue = reader.string();
          break;
        case 5:
          message.subProperties.push(
            ResourceProperty.decode(reader, reader.uint32())
          );
          break;
        case 6:
          const entry6 = ResourceProperty_AttributesEntry.decode(
            reader,
            reader.uint32()
          );
          if (entry6.value !== undefined) {
            message.attributes[entry6.key] = entry6.value;
          }
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ResourceProperty {
    const message = { ...baseResourceProperty } as ResourceProperty;
    message.name =
      object.name !== undefined && object.name !== null
        ? String(object.name)
        : "";
    message.ptype =
      object.ptype !== undefined && object.ptype !== null
        ? String(object.ptype)
        : "";
    message.jsonValue =
      object.jsonValue !== undefined && object.jsonValue !== null
        ? String(object.jsonValue)
        : undefined;
    message.subProperties = (object.subProperties ?? []).map((e: any) =>
      ResourceProperty.fromJSON(e)
    );
    message.attributes = Object.entries(object.attributes ?? {}).reduce<{
      [key: string]: string;
    }>((acc, [key, value]) => {
      acc[key] = String(value);
      return acc;
    }, {});
    return message;
  },

  toJSON(message: ResourceProperty): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.ptype !== undefined && (obj.ptype = message.ptype);
    message.jsonValue !== undefined && (obj.jsonValue = message.jsonValue);
    if (message.subProperties) {
      obj.subProperties = message.subProperties.map((e) =>
        e ? ResourceProperty.toJSON(e) : undefined
      );
    } else {
      obj.subProperties = [];
    }
    obj.attributes = {};
    if (message.attributes) {
      Object.entries(message.attributes).forEach(([k, v]) => {
        obj.attributes[k] = v;
      });
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ResourceProperty>, I>>(
    object: I
  ): ResourceProperty {
    const message = { ...baseResourceProperty } as ResourceProperty;
    message.name = object.name ?? "";
    message.ptype = object.ptype ?? "";
    message.jsonValue = object.jsonValue ?? undefined;
    message.subProperties =
      object.subProperties?.map((e) => ResourceProperty.fromPartial(e)) || [];
    message.attributes = Object.entries(object.attributes ?? {}).reduce<{
      [key: string]: string;
    }>((acc, [key, value]) => {
      if (value !== undefined) {
        acc[key] = String(value);
      }
      return acc;
    }, {});
    return message;
  },
};

const baseResourceProperty_AttributesEntry: object = { key: "", value: "" };

export const ResourceProperty_AttributesEntry = {
  encode(
    message: ResourceProperty_AttributesEntry,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.key !== "") {
      writer.uint32(10).string(message.key);
    }
    if (message.value !== "") {
      writer.uint32(18).string(message.value);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ResourceProperty_AttributesEntry {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseResourceProperty_AttributesEntry,
    } as ResourceProperty_AttributesEntry;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.key = reader.string();
          break;
        case 2:
          message.value = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ResourceProperty_AttributesEntry {
    const message = {
      ...baseResourceProperty_AttributesEntry,
    } as ResourceProperty_AttributesEntry;
    message.key =
      object.key !== undefined && object.key !== null ? String(object.key) : "";
    message.value =
      object.value !== undefined && object.value !== null
        ? String(object.value)
        : "";
    return message;
  },

  toJSON(message: ResourceProperty_AttributesEntry): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = message.key);
    message.value !== undefined && (obj.value = message.value);
    return obj;
  },

  fromPartial<
    I extends Exact<DeepPartial<ResourceProperty_AttributesEntry>, I>
  >(object: I): ResourceProperty_AttributesEntry {
    const message = {
      ...baseResourceProperty_AttributesEntry,
    } as ResourceProperty_AttributesEntry;
    message.key = object.key ?? "";
    message.value = object.value ?? "";
    return message;
  },
};

const baseUpdateResourcePropertiesRequest: object = { id: "", version: 0 };

export const UpdateResourcePropertiesRequest = {
  encode(
    message: UpdateResourcePropertiesRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.version !== 0) {
      writer.uint32(16).uint32(message.version);
    }
    for (const v of message.propertyUpdates) {
      ResourcePropertyUpdate.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): UpdateResourcePropertiesRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseUpdateResourcePropertiesRequest,
    } as UpdateResourcePropertiesRequest;
    message.propertyUpdates = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.version = reader.uint32();
          break;
        case 3:
          message.propertyUpdates.push(
            ResourcePropertyUpdate.decode(reader, reader.uint32())
          );
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): UpdateResourcePropertiesRequest {
    const message = {
      ...baseUpdateResourcePropertiesRequest,
    } as UpdateResourcePropertiesRequest;
    message.id =
      object.id !== undefined && object.id !== null ? String(object.id) : "";
    message.version =
      object.version !== undefined && object.version !== null
        ? Number(object.version)
        : 0;
    message.propertyUpdates = (object.propertyUpdates ?? []).map((e: any) =>
      ResourcePropertyUpdate.fromJSON(e)
    );
    return message;
  },

  toJSON(message: UpdateResourcePropertiesRequest): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.version !== undefined &&
      (obj.version = Math.round(message.version));
    if (message.propertyUpdates) {
      obj.propertyUpdates = message.propertyUpdates.map((e) =>
        e ? ResourcePropertyUpdate.toJSON(e) : undefined
      );
    } else {
      obj.propertyUpdates = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<UpdateResourcePropertiesRequest>, I>>(
    object: I
  ): UpdateResourcePropertiesRequest {
    const message = {
      ...baseUpdateResourcePropertiesRequest,
    } as UpdateResourcePropertiesRequest;
    message.id = object.id ?? "";
    message.version = object.version ?? 0;
    message.propertyUpdates =
      object.propertyUpdates?.map((e) =>
        ResourcePropertyUpdate.fromPartial(e)
      ) || [];
    return message;
  },
};

const baseUpdateResourcePropertiesResponse: object = {};

export const UpdateResourcePropertiesResponse = {
  encode(
    _: UpdateResourcePropertiesResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): UpdateResourcePropertiesResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseUpdateResourcePropertiesResponse,
    } as UpdateResourcePropertiesResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): UpdateResourcePropertiesResponse {
    const message = {
      ...baseUpdateResourcePropertiesResponse,
    } as UpdateResourcePropertiesResponse;
    return message;
  },

  toJSON(_: UpdateResourcePropertiesResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<
    I extends Exact<DeepPartial<UpdateResourcePropertiesResponse>, I>
  >(_: I): UpdateResourcePropertiesResponse {
    const message = {
      ...baseUpdateResourcePropertiesResponse,
    } as UpdateResourcePropertiesResponse;
    return message;
  },
};

const baseResourcePropertyUpdate: object = { name: "", jsonValue: "" };

export const ResourcePropertyUpdate = {
  encode(
    message: ResourcePropertyUpdate,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.jsonValue !== "") {
      writer.uint32(18).string(message.jsonValue);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ResourcePropertyUpdate {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseResourcePropertyUpdate } as ResourcePropertyUpdate;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.jsonValue = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ResourcePropertyUpdate {
    const message = { ...baseResourcePropertyUpdate } as ResourcePropertyUpdate;
    message.name =
      object.name !== undefined && object.name !== null
        ? String(object.name)
        : "";
    message.jsonValue =
      object.jsonValue !== undefined && object.jsonValue !== null
        ? String(object.jsonValue)
        : "";
    return message;
  },

  toJSON(message: ResourcePropertyUpdate): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.jsonValue !== undefined && (obj.jsonValue = message.jsonValue);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ResourcePropertyUpdate>, I>>(
    object: I
  ): ResourcePropertyUpdate {
    const message = { ...baseResourcePropertyUpdate } as ResourcePropertyUpdate;
    message.name = object.name ?? "";
    message.jsonValue = object.jsonValue ?? "";
    return message;
  },
};

const baseDeleteArrayElementRequest: object = {
  resourceId: "",
  arrayPath: "",
  indices: 0,
};

export const DeleteArrayElementRequest = {
  encode(
    message: DeleteArrayElementRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.resourceId !== "") {
      writer.uint32(10).string(message.resourceId);
    }
    if (message.arrayPath !== "") {
      writer.uint32(18).string(message.arrayPath);
    }
    writer.uint32(26).fork();
    for (const v of message.indices) {
      writer.uint64(v);
    }
    writer.ldelim();
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): DeleteArrayElementRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseDeleteArrayElementRequest,
    } as DeleteArrayElementRequest;
    message.indices = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.resourceId = reader.string();
          break;
        case 2:
          message.arrayPath = reader.string();
          break;
        case 3:
          if ((tag & 7) === 2) {
            const end2 = reader.uint32() + reader.pos;
            while (reader.pos < end2) {
              message.indices.push(longToNumber(reader.uint64() as Long));
            }
          } else {
            message.indices.push(longToNumber(reader.uint64() as Long));
          }
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): DeleteArrayElementRequest {
    const message = {
      ...baseDeleteArrayElementRequest,
    } as DeleteArrayElementRequest;
    message.resourceId =
      object.resourceId !== undefined && object.resourceId !== null
        ? String(object.resourceId)
        : "";
    message.arrayPath =
      object.arrayPath !== undefined && object.arrayPath !== null
        ? String(object.arrayPath)
        : "";
    message.indices = (object.indices ?? []).map((e: any) => Number(e));
    return message;
  },

  toJSON(message: DeleteArrayElementRequest): unknown {
    const obj: any = {};
    message.resourceId !== undefined && (obj.resourceId = message.resourceId);
    message.arrayPath !== undefined && (obj.arrayPath = message.arrayPath);
    if (message.indices) {
      obj.indices = message.indices.map((e) => Math.round(e));
    } else {
      obj.indices = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<DeleteArrayElementRequest>, I>>(
    object: I
  ): DeleteArrayElementRequest {
    const message = {
      ...baseDeleteArrayElementRequest,
    } as DeleteArrayElementRequest;
    message.resourceId = object.resourceId ?? "";
    message.arrayPath = object.arrayPath ?? "";
    message.indices = object.indices?.map((e) => e) || [];
    return message;
  },
};

const baseDeleteArrayElementResponse: object = {};

export const DeleteArrayElementResponse = {
  encode(
    _: DeleteArrayElementResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): DeleteArrayElementResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseDeleteArrayElementResponse,
    } as DeleteArrayElementResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): DeleteArrayElementResponse {
    const message = {
      ...baseDeleteArrayElementResponse,
    } as DeleteArrayElementResponse;
    return message;
  },

  toJSON(_: DeleteArrayElementResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<DeleteArrayElementResponse>, I>>(
    _: I
  ): DeleteArrayElementResponse {
    const message = {
      ...baseDeleteArrayElementResponse,
    } as DeleteArrayElementResponse;
    return message;
  },
};

const baseInsertNewArrayElementRequest: object = {
  resourceId: "",
  arrayPath: "",
  index: 0,
  jsonValue: "",
};

export const InsertNewArrayElementRequest = {
  encode(
    message: InsertNewArrayElementRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.resourceId !== "") {
      writer.uint32(10).string(message.resourceId);
    }
    if (message.arrayPath !== "") {
      writer.uint32(18).string(message.arrayPath);
    }
    if (message.index !== 0) {
      writer.uint32(24).uint64(message.index);
    }
    if (message.jsonValue !== "") {
      writer.uint32(34).string(message.jsonValue);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): InsertNewArrayElementRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseInsertNewArrayElementRequest,
    } as InsertNewArrayElementRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.resourceId = reader.string();
          break;
        case 2:
          message.arrayPath = reader.string();
          break;
        case 3:
          message.index = longToNumber(reader.uint64() as Long);
          break;
        case 4:
          message.jsonValue = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InsertNewArrayElementRequest {
    const message = {
      ...baseInsertNewArrayElementRequest,
    } as InsertNewArrayElementRequest;
    message.resourceId =
      object.resourceId !== undefined && object.resourceId !== null
        ? String(object.resourceId)
        : "";
    message.arrayPath =
      object.arrayPath !== undefined && object.arrayPath !== null
        ? String(object.arrayPath)
        : "";
    message.index =
      object.index !== undefined && object.index !== null
        ? Number(object.index)
        : 0;
    message.jsonValue =
      object.jsonValue !== undefined && object.jsonValue !== null
        ? String(object.jsonValue)
        : "";
    return message;
  },

  toJSON(message: InsertNewArrayElementRequest): unknown {
    const obj: any = {};
    message.resourceId !== undefined && (obj.resourceId = message.resourceId);
    message.arrayPath !== undefined && (obj.arrayPath = message.arrayPath);
    message.index !== undefined && (obj.index = Math.round(message.index));
    message.jsonValue !== undefined && (obj.jsonValue = message.jsonValue);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertNewArrayElementRequest>, I>>(
    object: I
  ): InsertNewArrayElementRequest {
    const message = {
      ...baseInsertNewArrayElementRequest,
    } as InsertNewArrayElementRequest;
    message.resourceId = object.resourceId ?? "";
    message.arrayPath = object.arrayPath ?? "";
    message.index = object.index ?? 0;
    message.jsonValue = object.jsonValue ?? "";
    return message;
  },
};

const baseInsertNewArrayElementResponse: object = {};

export const InsertNewArrayElementResponse = {
  encode(
    _: InsertNewArrayElementResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): InsertNewArrayElementResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseInsertNewArrayElementResponse,
    } as InsertNewArrayElementResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): InsertNewArrayElementResponse {
    const message = {
      ...baseInsertNewArrayElementResponse,
    } as InsertNewArrayElementResponse;
    return message;
  },

  toJSON(_: InsertNewArrayElementResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertNewArrayElementResponse>, I>>(
    _: I
  ): InsertNewArrayElementResponse {
    const message = {
      ...baseInsertNewArrayElementResponse,
    } as InsertNewArrayElementResponse;
    return message;
  },
};

const baseReorderArrayElementRequest: object = {
  resourceId: "",
  arrayPath: "",
  oldIndex: 0,
  newIndex: 0,
};

export const ReorderArrayElementRequest = {
  encode(
    message: ReorderArrayElementRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.resourceId !== "") {
      writer.uint32(10).string(message.resourceId);
    }
    if (message.arrayPath !== "") {
      writer.uint32(18).string(message.arrayPath);
    }
    if (message.oldIndex !== 0) {
      writer.uint32(24).uint64(message.oldIndex);
    }
    if (message.newIndex !== 0) {
      writer.uint32(32).uint64(message.newIndex);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ReorderArrayElementRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseReorderArrayElementRequest,
    } as ReorderArrayElementRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.resourceId = reader.string();
          break;
        case 2:
          message.arrayPath = reader.string();
          break;
        case 3:
          message.oldIndex = longToNumber(reader.uint64() as Long);
          break;
        case 4:
          message.newIndex = longToNumber(reader.uint64() as Long);
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReorderArrayElementRequest {
    const message = {
      ...baseReorderArrayElementRequest,
    } as ReorderArrayElementRequest;
    message.resourceId =
      object.resourceId !== undefined && object.resourceId !== null
        ? String(object.resourceId)
        : "";
    message.arrayPath =
      object.arrayPath !== undefined && object.arrayPath !== null
        ? String(object.arrayPath)
        : "";
    message.oldIndex =
      object.oldIndex !== undefined && object.oldIndex !== null
        ? Number(object.oldIndex)
        : 0;
    message.newIndex =
      object.newIndex !== undefined && object.newIndex !== null
        ? Number(object.newIndex)
        : 0;
    return message;
  },

  toJSON(message: ReorderArrayElementRequest): unknown {
    const obj: any = {};
    message.resourceId !== undefined && (obj.resourceId = message.resourceId);
    message.arrayPath !== undefined && (obj.arrayPath = message.arrayPath);
    message.oldIndex !== undefined &&
      (obj.oldIndex = Math.round(message.oldIndex));
    message.newIndex !== undefined &&
      (obj.newIndex = Math.round(message.newIndex));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ReorderArrayElementRequest>, I>>(
    object: I
  ): ReorderArrayElementRequest {
    const message = {
      ...baseReorderArrayElementRequest,
    } as ReorderArrayElementRequest;
    message.resourceId = object.resourceId ?? "";
    message.arrayPath = object.arrayPath ?? "";
    message.oldIndex = object.oldIndex ?? 0;
    message.newIndex = object.newIndex ?? 0;
    return message;
  },
};

const baseReorderArrayElementResponse: object = {};

export const ReorderArrayElementResponse = {
  encode(
    _: ReorderArrayElementResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ReorderArrayElementResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseReorderArrayElementResponse,
    } as ReorderArrayElementResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(_: any): ReorderArrayElementResponse {
    const message = {
      ...baseReorderArrayElementResponse,
    } as ReorderArrayElementResponse;
    return message;
  },

  toJSON(_: ReorderArrayElementResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ReorderArrayElementResponse>, I>>(
    _: I
  ): ReorderArrayElementResponse {
    const message = {
      ...baseReorderArrayElementResponse,
    } as ReorderArrayElementResponse;
    return message;
  },
};

export interface PropertyInspector {
  getResourceProperties(
    request: DeepPartial<GetResourcePropertiesRequest>,
    metadata?: grpc.Metadata
  ): Promise<GetResourcePropertiesResponse>;
  updateResourceProperties(
    request: DeepPartial<UpdateResourcePropertiesRequest>,
    metadata?: grpc.Metadata
  ): Promise<UpdateResourcePropertiesResponse>;
  deleteArrayElement(
    request: DeepPartial<DeleteArrayElementRequest>,
    metadata?: grpc.Metadata
  ): Promise<DeleteArrayElementResponse>;
  insertNewArrayElement(
    request: DeepPartial<InsertNewArrayElementRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertNewArrayElementResponse>;
  reorderArrayElement(
    request: DeepPartial<ReorderArrayElementRequest>,
    metadata?: grpc.Metadata
  ): Promise<ReorderArrayElementResponse>;
}

export class PropertyInspectorClientImpl implements PropertyInspector {
  private readonly rpc: Rpc;

  constructor(rpc: Rpc) {
    this.rpc = rpc;
    this.getResourceProperties = this.getResourceProperties.bind(this);
    this.updateResourceProperties = this.updateResourceProperties.bind(this);
    this.deleteArrayElement = this.deleteArrayElement.bind(this);
    this.insertNewArrayElement = this.insertNewArrayElement.bind(this);
    this.reorderArrayElement = this.reorderArrayElement.bind(this);
  }

  getResourceProperties(
    request: DeepPartial<GetResourcePropertiesRequest>,
    metadata?: grpc.Metadata
  ): Promise<GetResourcePropertiesResponse> {
    return this.rpc.unary(
      PropertyInspectorGetResourcePropertiesDesc,
      GetResourcePropertiesRequest.fromPartial(request),
      metadata
    );
  }

  updateResourceProperties(
    request: DeepPartial<UpdateResourcePropertiesRequest>,
    metadata?: grpc.Metadata
  ): Promise<UpdateResourcePropertiesResponse> {
    return this.rpc.unary(
      PropertyInspectorUpdateResourcePropertiesDesc,
      UpdateResourcePropertiesRequest.fromPartial(request),
      metadata
    );
  }

  deleteArrayElement(
    request: DeepPartial<DeleteArrayElementRequest>,
    metadata?: grpc.Metadata
  ): Promise<DeleteArrayElementResponse> {
    return this.rpc.unary(
      PropertyInspectorDeleteArrayElementDesc,
      DeleteArrayElementRequest.fromPartial(request),
      metadata
    );
  }

  insertNewArrayElement(
    request: DeepPartial<InsertNewArrayElementRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertNewArrayElementResponse> {
    return this.rpc.unary(
      PropertyInspectorInsertNewArrayElementDesc,
      InsertNewArrayElementRequest.fromPartial(request),
      metadata
    );
  }

  reorderArrayElement(
    request: DeepPartial<ReorderArrayElementRequest>,
    metadata?: grpc.Metadata
  ): Promise<ReorderArrayElementResponse> {
    return this.rpc.unary(
      PropertyInspectorReorderArrayElementDesc,
      ReorderArrayElementRequest.fromPartial(request),
      metadata
    );
  }
}

export const PropertyInspectorDesc = {
  serviceName: "property_inspector.PropertyInspector",
};

export const PropertyInspectorGetResourcePropertiesDesc: UnaryMethodDefinitionish =
  {
    methodName: "GetResourceProperties",
    service: PropertyInspectorDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return GetResourcePropertiesRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...GetResourcePropertiesResponse.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PropertyInspectorUpdateResourcePropertiesDesc: UnaryMethodDefinitionish =
  {
    methodName: "UpdateResourceProperties",
    service: PropertyInspectorDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return UpdateResourcePropertiesRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...UpdateResourcePropertiesResponse.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PropertyInspectorDeleteArrayElementDesc: UnaryMethodDefinitionish =
  {
    methodName: "DeleteArrayElement",
    service: PropertyInspectorDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return DeleteArrayElementRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...DeleteArrayElementResponse.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PropertyInspectorInsertNewArrayElementDesc: UnaryMethodDefinitionish =
  {
    methodName: "InsertNewArrayElement",
    service: PropertyInspectorDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return InsertNewArrayElementRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...InsertNewArrayElementResponse.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PropertyInspectorReorderArrayElementDesc: UnaryMethodDefinitionish =
  {
    methodName: "ReorderArrayElement",
    service: PropertyInspectorDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return ReorderArrayElementRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...ReorderArrayElementResponse.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

interface UnaryMethodDefinitionishR
  extends grpc.UnaryMethodDefinition<any, any> {
  requestStream: any;
  responseStream: any;
}

type UnaryMethodDefinitionish = UnaryMethodDefinitionishR;

interface Rpc {
  unary<T extends UnaryMethodDefinitionish>(
    methodDesc: T,
    request: any,
    metadata: grpc.Metadata | undefined
  ): Promise<any>;
}

export class GrpcWebImpl {
  private host: string;
  private options: {
    transport?: grpc.TransportFactory;

    debug?: boolean;
    metadata?: grpc.Metadata;
  };

  constructor(
    host: string,
    options: {
      transport?: grpc.TransportFactory;

      debug?: boolean;
      metadata?: grpc.Metadata;
    }
  ) {
    this.host = host;
    this.options = options;
  }

  unary<T extends UnaryMethodDefinitionish>(
    methodDesc: T,
    _request: any,
    metadata: grpc.Metadata | undefined
  ): Promise<any> {
    const request = { ..._request, ...methodDesc.requestType };
    const maybeCombinedMetadata =
      metadata && this.options.metadata
        ? new BrowserHeaders({
            ...this.options?.metadata.headersMap,
            ...metadata?.headersMap,
          })
        : metadata || this.options.metadata;
    return new Promise((resolve, reject) => {
      grpc.unary(methodDesc, {
        request,
        host: this.host,
        metadata: maybeCombinedMetadata,
        transport: this.options.transport,
        debug: this.options.debug,
        onEnd: function (response) {
          if (response.status === grpc.Code.OK) {
            resolve(response.message);
          } else {
            const err = new Error(response.statusMessage) as any;
            err.code = response.status;
            err.metadata = response.trailers;
            reject(err);
          }
        },
      });
    });
  }
}

declare var self: any | undefined;
declare var window: any | undefined;
declare var global: any | undefined;
var globalThis: any = (() => {
  if (typeof globalThis !== "undefined") return globalThis;
  if (typeof self !== "undefined") return self;
  if (typeof window !== "undefined") return window;
  if (typeof global !== "undefined") return global;
  throw "Unable to locate global object";
})();

type Builtin =
  | Date
  | Function
  | Uint8Array
  | string
  | number
  | boolean
  | undefined;

export type DeepPartial<T> = T extends Builtin
  ? T
  : T extends Array<infer U>
  ? Array<DeepPartial<U>>
  : T extends ReadonlyArray<infer U>
  ? ReadonlyArray<DeepPartial<U>>
  : T extends {}
  ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type Exact<P, I extends P> = P extends Builtin
  ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & Record<
        Exclude<keyof I, KeysOfUnion<P>>,
        never
      >;

function longToNumber(long: Long): number {
  if (long.gt(Number.MAX_SAFE_INTEGER)) {
    throw new globalThis.Error("Value is larger than Number.MAX_SAFE_INTEGER");
  }
  return long.toNumber();
}

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}
