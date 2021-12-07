/* eslint-disable */
import Long from "long";
import { grpc } from "@improbable-eng/grpc-web";
import _m0 from "protobufjs/minimal";
import { BrowserHeaders } from "browser-headers";

export const protobufPackage = "editor";

export interface UndoTransactionRequest {
  id: number;
}

export interface UndoTransactionResponse {
  id: number;
}

export interface RedoTransactionRequest {
  id: number;
}

export interface RedoTransactionResponse {
  id: number;
}

export interface SearchResourcesRequest {
  searchToken: string;
}

export interface SearchResourcesResponse {
  nextSearchToken: string;
  total: number;
  resourceDescriptions: ResourceDescription[];
}

export interface ResourceDescription {
  id: string;
  path: string;
  version: number;
}

export interface GetResourcePropertiesRequest {
  id: string;
}

export interface GetResourcePropertiesResponse {
  description: ResourceDescription | undefined;
  properties: ResourceProperty[];
}

export interface ResourceProperty {
  name: string;
  ptype: string;
  defaultValue: Uint8Array;
  value: Uint8Array;
  group: string;
}

export interface UpdateResourcePropertiesRequest {
  id: string;
  version: number;
  propertyUpdates: ResourcePropertyUpdate[];
}

export interface UpdateResourcePropertiesResponse {
  version: number;
  updatedProperties: ResourcePropertyUpdate[];
}

export interface ResourcePropertyUpdate {
  name: string;
  value: Uint8Array;
}

const baseUndoTransactionRequest: object = { id: 0 };

export const UndoTransactionRequest = {
  encode(
    message: UndoTransactionRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== 0) {
      writer.uint32(8).int32(message.id);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): UndoTransactionRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseUndoTransactionRequest } as UndoTransactionRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): UndoTransactionRequest {
    const message = { ...baseUndoTransactionRequest } as UndoTransactionRequest;
    message.id =
      object.id !== undefined && object.id !== null ? Number(object.id) : 0;
    return message;
  },

  toJSON(message: UndoTransactionRequest): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<UndoTransactionRequest>, I>>(
    object: I
  ): UndoTransactionRequest {
    const message = { ...baseUndoTransactionRequest } as UndoTransactionRequest;
    message.id = object.id ?? 0;
    return message;
  },
};

const baseUndoTransactionResponse: object = { id: 0 };

export const UndoTransactionResponse = {
  encode(
    message: UndoTransactionResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== 0) {
      writer.uint32(8).int32(message.id);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): UndoTransactionResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseUndoTransactionResponse,
    } as UndoTransactionResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): UndoTransactionResponse {
    const message = {
      ...baseUndoTransactionResponse,
    } as UndoTransactionResponse;
    message.id =
      object.id !== undefined && object.id !== null ? Number(object.id) : 0;
    return message;
  },

  toJSON(message: UndoTransactionResponse): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<UndoTransactionResponse>, I>>(
    object: I
  ): UndoTransactionResponse {
    const message = {
      ...baseUndoTransactionResponse,
    } as UndoTransactionResponse;
    message.id = object.id ?? 0;
    return message;
  },
};

const baseRedoTransactionRequest: object = { id: 0 };

export const RedoTransactionRequest = {
  encode(
    message: RedoTransactionRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== 0) {
      writer.uint32(8).int32(message.id);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): RedoTransactionRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseRedoTransactionRequest } as RedoTransactionRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): RedoTransactionRequest {
    const message = { ...baseRedoTransactionRequest } as RedoTransactionRequest;
    message.id =
      object.id !== undefined && object.id !== null ? Number(object.id) : 0;
    return message;
  },

  toJSON(message: RedoTransactionRequest): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<RedoTransactionRequest>, I>>(
    object: I
  ): RedoTransactionRequest {
    const message = { ...baseRedoTransactionRequest } as RedoTransactionRequest;
    message.id = object.id ?? 0;
    return message;
  },
};

const baseRedoTransactionResponse: object = { id: 0 };

export const RedoTransactionResponse = {
  encode(
    message: RedoTransactionResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== 0) {
      writer.uint32(8).int32(message.id);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): RedoTransactionResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseRedoTransactionResponse,
    } as RedoTransactionResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): RedoTransactionResponse {
    const message = {
      ...baseRedoTransactionResponse,
    } as RedoTransactionResponse;
    message.id =
      object.id !== undefined && object.id !== null ? Number(object.id) : 0;
    return message;
  },

  toJSON(message: RedoTransactionResponse): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<RedoTransactionResponse>, I>>(
    object: I
  ): RedoTransactionResponse {
    const message = {
      ...baseRedoTransactionResponse,
    } as RedoTransactionResponse;
    message.id = object.id ?? 0;
    return message;
  },
};

const baseSearchResourcesRequest: object = { searchToken: "" };

export const SearchResourcesRequest = {
  encode(
    message: SearchResourcesRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.searchToken !== "") {
      writer.uint32(10).string(message.searchToken);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): SearchResourcesRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseSearchResourcesRequest } as SearchResourcesRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.searchToken = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): SearchResourcesRequest {
    const message = { ...baseSearchResourcesRequest } as SearchResourcesRequest;
    message.searchToken =
      object.searchToken !== undefined && object.searchToken !== null
        ? String(object.searchToken)
        : "";
    return message;
  },

  toJSON(message: SearchResourcesRequest): unknown {
    const obj: any = {};
    message.searchToken !== undefined &&
      (obj.searchToken = message.searchToken);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<SearchResourcesRequest>, I>>(
    object: I
  ): SearchResourcesRequest {
    const message = { ...baseSearchResourcesRequest } as SearchResourcesRequest;
    message.searchToken = object.searchToken ?? "";
    return message;
  },
};

const baseSearchResourcesResponse: object = { nextSearchToken: "", total: 0 };

export const SearchResourcesResponse = {
  encode(
    message: SearchResourcesResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.nextSearchToken !== "") {
      writer.uint32(10).string(message.nextSearchToken);
    }
    if (message.total !== 0) {
      writer.uint32(16).uint64(message.total);
    }
    for (const v of message.resourceDescriptions) {
      ResourceDescription.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): SearchResourcesResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseSearchResourcesResponse,
    } as SearchResourcesResponse;
    message.resourceDescriptions = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.nextSearchToken = reader.string();
          break;
        case 2:
          message.total = longToNumber(reader.uint64() as Long);
          break;
        case 3:
          message.resourceDescriptions.push(
            ResourceDescription.decode(reader, reader.uint32())
          );
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): SearchResourcesResponse {
    const message = {
      ...baseSearchResourcesResponse,
    } as SearchResourcesResponse;
    message.nextSearchToken =
      object.nextSearchToken !== undefined && object.nextSearchToken !== null
        ? String(object.nextSearchToken)
        : "";
    message.total =
      object.total !== undefined && object.total !== null
        ? Number(object.total)
        : 0;
    message.resourceDescriptions = (object.resourceDescriptions ?? []).map(
      (e: any) => ResourceDescription.fromJSON(e)
    );
    return message;
  },

  toJSON(message: SearchResourcesResponse): unknown {
    const obj: any = {};
    message.nextSearchToken !== undefined &&
      (obj.nextSearchToken = message.nextSearchToken);
    message.total !== undefined && (obj.total = message.total);
    if (message.resourceDescriptions) {
      obj.resourceDescriptions = message.resourceDescriptions.map((e) =>
        e ? ResourceDescription.toJSON(e) : undefined
      );
    } else {
      obj.resourceDescriptions = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<SearchResourcesResponse>, I>>(
    object: I
  ): SearchResourcesResponse {
    const message = {
      ...baseSearchResourcesResponse,
    } as SearchResourcesResponse;
    message.nextSearchToken = object.nextSearchToken ?? "";
    message.total = object.total ?? 0;
    message.resourceDescriptions =
      object.resourceDescriptions?.map((e) =>
        ResourceDescription.fromPartial(e)
      ) || [];
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
    message.version !== undefined && (obj.version = message.version);
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

const baseResourceProperty: object = { name: "", ptype: "", group: "" };

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
    if (message.defaultValue.length !== 0) {
      writer.uint32(26).bytes(message.defaultValue);
    }
    if (message.value.length !== 0) {
      writer.uint32(34).bytes(message.value);
    }
    if (message.group !== "") {
      writer.uint32(42).string(message.group);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ResourceProperty {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseResourceProperty } as ResourceProperty;
    message.defaultValue = new Uint8Array();
    message.value = new Uint8Array();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.ptype = reader.string();
          break;
        case 3:
          message.defaultValue = reader.bytes();
          break;
        case 4:
          message.value = reader.bytes();
          break;
        case 5:
          message.group = reader.string();
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
    message.defaultValue =
      object.defaultValue !== undefined && object.defaultValue !== null
        ? bytesFromBase64(object.defaultValue)
        : new Uint8Array();
    message.value =
      object.value !== undefined && object.value !== null
        ? bytesFromBase64(object.value)
        : new Uint8Array();
    message.group =
      object.group !== undefined && object.group !== null
        ? String(object.group)
        : "";
    return message;
  },

  toJSON(message: ResourceProperty): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.ptype !== undefined && (obj.ptype = message.ptype);
    message.defaultValue !== undefined &&
      (obj.defaultValue = base64FromBytes(
        message.defaultValue !== undefined
          ? message.defaultValue
          : new Uint8Array()
      ));
    message.value !== undefined &&
      (obj.value = base64FromBytes(
        message.value !== undefined ? message.value : new Uint8Array()
      ));
    message.group !== undefined && (obj.group = message.group);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ResourceProperty>, I>>(
    object: I
  ): ResourceProperty {
    const message = { ...baseResourceProperty } as ResourceProperty;
    message.name = object.name ?? "";
    message.ptype = object.ptype ?? "";
    message.defaultValue = object.defaultValue ?? new Uint8Array();
    message.value = object.value ?? new Uint8Array();
    message.group = object.group ?? "";
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
    message.version !== undefined && (obj.version = message.version);
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

const baseUpdateResourcePropertiesResponse: object = { version: 0 };

export const UpdateResourcePropertiesResponse = {
  encode(
    message: UpdateResourcePropertiesResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.version !== 0) {
      writer.uint32(8).uint32(message.version);
    }
    for (const v of message.updatedProperties) {
      ResourcePropertyUpdate.encode(v!, writer.uint32(18).fork()).ldelim();
    }
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
    message.updatedProperties = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.version = reader.uint32();
          break;
        case 2:
          message.updatedProperties.push(
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

  fromJSON(object: any): UpdateResourcePropertiesResponse {
    const message = {
      ...baseUpdateResourcePropertiesResponse,
    } as UpdateResourcePropertiesResponse;
    message.version =
      object.version !== undefined && object.version !== null
        ? Number(object.version)
        : 0;
    message.updatedProperties = (object.updatedProperties ?? []).map((e: any) =>
      ResourcePropertyUpdate.fromJSON(e)
    );
    return message;
  },

  toJSON(message: UpdateResourcePropertiesResponse): unknown {
    const obj: any = {};
    message.version !== undefined && (obj.version = message.version);
    if (message.updatedProperties) {
      obj.updatedProperties = message.updatedProperties.map((e) =>
        e ? ResourcePropertyUpdate.toJSON(e) : undefined
      );
    } else {
      obj.updatedProperties = [];
    }
    return obj;
  },

  fromPartial<
    I extends Exact<DeepPartial<UpdateResourcePropertiesResponse>, I>
  >(object: I): UpdateResourcePropertiesResponse {
    const message = {
      ...baseUpdateResourcePropertiesResponse,
    } as UpdateResourcePropertiesResponse;
    message.version = object.version ?? 0;
    message.updatedProperties =
      object.updatedProperties?.map((e) =>
        ResourcePropertyUpdate.fromPartial(e)
      ) || [];
    return message;
  },
};

const baseResourcePropertyUpdate: object = { name: "" };

export const ResourcePropertyUpdate = {
  encode(
    message: ResourcePropertyUpdate,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.value.length !== 0) {
      writer.uint32(18).bytes(message.value);
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
    message.value = new Uint8Array();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.value = reader.bytes();
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
    message.value =
      object.value !== undefined && object.value !== null
        ? bytesFromBase64(object.value)
        : new Uint8Array();
    return message;
  },

  toJSON(message: ResourcePropertyUpdate): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.value !== undefined &&
      (obj.value = base64FromBytes(
        message.value !== undefined ? message.value : new Uint8Array()
      ));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ResourcePropertyUpdate>, I>>(
    object: I
  ): ResourcePropertyUpdate {
    const message = { ...baseResourcePropertyUpdate } as ResourcePropertyUpdate;
    message.name = object.name ?? "";
    message.value = object.value ?? new Uint8Array();
    return message;
  },
};

export interface Editor {
  searchResources(
    request: DeepPartial<SearchResourcesRequest>,
    metadata?: grpc.Metadata
  ): Promise<SearchResourcesResponse>;
  undoTransaction(
    request: DeepPartial<UndoTransactionRequest>,
    metadata?: grpc.Metadata
  ): Promise<UndoTransactionResponse>;
  redoTransaction(
    request: DeepPartial<RedoTransactionRequest>,
    metadata?: grpc.Metadata
  ): Promise<RedoTransactionResponse>;
  getResourceProperties(
    request: DeepPartial<GetResourcePropertiesRequest>,
    metadata?: grpc.Metadata
  ): Promise<GetResourcePropertiesResponse>;
  updateResourceProperties(
    request: DeepPartial<UpdateResourcePropertiesRequest>,
    metadata?: grpc.Metadata
  ): Promise<UpdateResourcePropertiesResponse>;
}

export class EditorClientImpl implements Editor {
  private readonly rpc: Rpc;

  constructor(rpc: Rpc) {
    this.rpc = rpc;
    this.searchResources = this.searchResources.bind(this);
    this.undoTransaction = this.undoTransaction.bind(this);
    this.redoTransaction = this.redoTransaction.bind(this);
    this.getResourceProperties = this.getResourceProperties.bind(this);
    this.updateResourceProperties = this.updateResourceProperties.bind(this);
  }

  searchResources(
    request: DeepPartial<SearchResourcesRequest>,
    metadata?: grpc.Metadata
  ): Promise<SearchResourcesResponse> {
    return this.rpc.unary(
      EditorSearchResourcesDesc,
      SearchResourcesRequest.fromPartial(request),
      metadata
    );
  }

  undoTransaction(
    request: DeepPartial<UndoTransactionRequest>,
    metadata?: grpc.Metadata
  ): Promise<UndoTransactionResponse> {
    return this.rpc.unary(
      EditorUndoTransactionDesc,
      UndoTransactionRequest.fromPartial(request),
      metadata
    );
  }

  redoTransaction(
    request: DeepPartial<RedoTransactionRequest>,
    metadata?: grpc.Metadata
  ): Promise<RedoTransactionResponse> {
    return this.rpc.unary(
      EditorRedoTransactionDesc,
      RedoTransactionRequest.fromPartial(request),
      metadata
    );
  }

  getResourceProperties(
    request: DeepPartial<GetResourcePropertiesRequest>,
    metadata?: grpc.Metadata
  ): Promise<GetResourcePropertiesResponse> {
    return this.rpc.unary(
      EditorGetResourcePropertiesDesc,
      GetResourcePropertiesRequest.fromPartial(request),
      metadata
    );
  }

  updateResourceProperties(
    request: DeepPartial<UpdateResourcePropertiesRequest>,
    metadata?: grpc.Metadata
  ): Promise<UpdateResourcePropertiesResponse> {
    return this.rpc.unary(
      EditorUpdateResourcePropertiesDesc,
      UpdateResourcePropertiesRequest.fromPartial(request),
      metadata
    );
  }
}

export const EditorDesc = {
  serviceName: "editor.Editor",
};

export const EditorSearchResourcesDesc: UnaryMethodDefinitionish = {
  methodName: "SearchResources",
  service: EditorDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return SearchResourcesRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...SearchResourcesResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const EditorUndoTransactionDesc: UnaryMethodDefinitionish = {
  methodName: "UndoTransaction",
  service: EditorDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return UndoTransactionRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...UndoTransactionResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const EditorRedoTransactionDesc: UnaryMethodDefinitionish = {
  methodName: "RedoTransaction",
  service: EditorDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return RedoTransactionRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...RedoTransactionResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const EditorGetResourcePropertiesDesc: UnaryMethodDefinitionish = {
  methodName: "GetResourceProperties",
  service: EditorDesc,
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

export const EditorUpdateResourcePropertiesDesc: UnaryMethodDefinitionish = {
  methodName: "UpdateResourceProperties",
  service: EditorDesc,
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

const atob: (b64: string) => string =
  globalThis.atob ||
  ((b64) => globalThis.Buffer.from(b64, "base64").toString("binary"));
function bytesFromBase64(b64: string): Uint8Array {
  const bin = atob(b64);
  const arr = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; ++i) {
    arr[i] = bin.charCodeAt(i);
  }
  return arr;
}

const btoa: (bin: string) => string =
  globalThis.btoa ||
  ((bin) => globalThis.Buffer.from(bin, "binary").toString("base64"));
function base64FromBytes(arr: Uint8Array): string {
  const bin: string[] = [];
  for (const byte of arr) {
    bin.push(String.fromCharCode(byte));
  }
  return btoa(bin.join(""));
}

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
