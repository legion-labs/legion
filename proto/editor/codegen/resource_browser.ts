/* eslint-disable */
import Long from "long";
import { grpc } from "@improbable-eng/grpc-web";
import _m0 from "protobufjs/minimal";
import { BrowserHeaders } from "browser-headers";

export const protobufPackage = "resource_browser";

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

export interface GetResourceTypeNamesRequest {}

export interface GetResourceTypeNamesResponse {
  resourceTypes: string[];
}

export interface CreateResourceRequest {
  resourceType: string;
  resourcePath: string;
}

export interface CreateResourceResponse {
  newId: string;
}

export interface ImportResourceRequest {
  resourceName: string;
  sharedFilePath: string;
}

export interface ImportResourceResponse {
  newId: string;
}

export interface DeleteResourceRequest {
  id: string;
}

export interface DeleteResourceResponse {}

export interface RenameResourceRequest {
  id: string;
  newPath: string;
}

export interface RenameResourceResponse {}

export interface CloneResourceRequest {
  sourceId: string;
  clonePath: string;
}

export interface CloneResourceResponse {
  newId: string;
}

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
    message.total !== undefined && (obj.total = Math.round(message.total));
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

const baseGetResourceTypeNamesRequest: object = {};

export const GetResourceTypeNamesRequest = {
  encode(
    _: GetResourceTypeNamesRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): GetResourceTypeNamesRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseGetResourceTypeNamesRequest,
    } as GetResourceTypeNamesRequest;
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

  fromJSON(_: any): GetResourceTypeNamesRequest {
    const message = {
      ...baseGetResourceTypeNamesRequest,
    } as GetResourceTypeNamesRequest;
    return message;
  },

  toJSON(_: GetResourceTypeNamesRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<GetResourceTypeNamesRequest>, I>>(
    _: I
  ): GetResourceTypeNamesRequest {
    const message = {
      ...baseGetResourceTypeNamesRequest,
    } as GetResourceTypeNamesRequest;
    return message;
  },
};

const baseGetResourceTypeNamesResponse: object = { resourceTypes: "" };

export const GetResourceTypeNamesResponse = {
  encode(
    message: GetResourceTypeNamesResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.resourceTypes) {
      writer.uint32(10).string(v!);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): GetResourceTypeNamesResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseGetResourceTypeNamesResponse,
    } as GetResourceTypeNamesResponse;
    message.resourceTypes = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.resourceTypes.push(reader.string());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): GetResourceTypeNamesResponse {
    const message = {
      ...baseGetResourceTypeNamesResponse,
    } as GetResourceTypeNamesResponse;
    message.resourceTypes = (object.resourceTypes ?? []).map((e: any) =>
      String(e)
    );
    return message;
  },

  toJSON(message: GetResourceTypeNamesResponse): unknown {
    const obj: any = {};
    if (message.resourceTypes) {
      obj.resourceTypes = message.resourceTypes.map((e) => e);
    } else {
      obj.resourceTypes = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<GetResourceTypeNamesResponse>, I>>(
    object: I
  ): GetResourceTypeNamesResponse {
    const message = {
      ...baseGetResourceTypeNamesResponse,
    } as GetResourceTypeNamesResponse;
    message.resourceTypes = object.resourceTypes?.map((e) => e) || [];
    return message;
  },
};

const baseCreateResourceRequest: object = {
  resourceType: "",
  resourcePath: "",
};

export const CreateResourceRequest = {
  encode(
    message: CreateResourceRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.resourceType !== "") {
      writer.uint32(10).string(message.resourceType);
    }
    if (message.resourcePath !== "") {
      writer.uint32(18).string(message.resourcePath);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CreateResourceRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCreateResourceRequest } as CreateResourceRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.resourceType = reader.string();
          break;
        case 2:
          message.resourcePath = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CreateResourceRequest {
    const message = { ...baseCreateResourceRequest } as CreateResourceRequest;
    message.resourceType =
      object.resourceType !== undefined && object.resourceType !== null
        ? String(object.resourceType)
        : "";
    message.resourcePath =
      object.resourcePath !== undefined && object.resourcePath !== null
        ? String(object.resourcePath)
        : "";
    return message;
  },

  toJSON(message: CreateResourceRequest): unknown {
    const obj: any = {};
    message.resourceType !== undefined &&
      (obj.resourceType = message.resourceType);
    message.resourcePath !== undefined &&
      (obj.resourcePath = message.resourcePath);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CreateResourceRequest>, I>>(
    object: I
  ): CreateResourceRequest {
    const message = { ...baseCreateResourceRequest } as CreateResourceRequest;
    message.resourceType = object.resourceType ?? "";
    message.resourcePath = object.resourcePath ?? "";
    return message;
  },
};

const baseCreateResourceResponse: object = { newId: "" };

export const CreateResourceResponse = {
  encode(
    message: CreateResourceResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.newId !== "") {
      writer.uint32(10).string(message.newId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CreateResourceResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCreateResourceResponse } as CreateResourceResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.newId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CreateResourceResponse {
    const message = { ...baseCreateResourceResponse } as CreateResourceResponse;
    message.newId =
      object.newId !== undefined && object.newId !== null
        ? String(object.newId)
        : "";
    return message;
  },

  toJSON(message: CreateResourceResponse): unknown {
    const obj: any = {};
    message.newId !== undefined && (obj.newId = message.newId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CreateResourceResponse>, I>>(
    object: I
  ): CreateResourceResponse {
    const message = { ...baseCreateResourceResponse } as CreateResourceResponse;
    message.newId = object.newId ?? "";
    return message;
  },
};

const baseImportResourceRequest: object = {
  resourceName: "",
  sharedFilePath: "",
};

export const ImportResourceRequest = {
  encode(
    message: ImportResourceRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.resourceName !== "") {
      writer.uint32(10).string(message.resourceName);
    }
    if (message.sharedFilePath !== "") {
      writer.uint32(18).string(message.sharedFilePath);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ImportResourceRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseImportResourceRequest } as ImportResourceRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.resourceName = reader.string();
          break;
        case 2:
          message.sharedFilePath = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ImportResourceRequest {
    const message = { ...baseImportResourceRequest } as ImportResourceRequest;
    message.resourceName =
      object.resourceName !== undefined && object.resourceName !== null
        ? String(object.resourceName)
        : "";
    message.sharedFilePath =
      object.sharedFilePath !== undefined && object.sharedFilePath !== null
        ? String(object.sharedFilePath)
        : "";
    return message;
  },

  toJSON(message: ImportResourceRequest): unknown {
    const obj: any = {};
    message.resourceName !== undefined &&
      (obj.resourceName = message.resourceName);
    message.sharedFilePath !== undefined &&
      (obj.sharedFilePath = message.sharedFilePath);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ImportResourceRequest>, I>>(
    object: I
  ): ImportResourceRequest {
    const message = { ...baseImportResourceRequest } as ImportResourceRequest;
    message.resourceName = object.resourceName ?? "";
    message.sharedFilePath = object.sharedFilePath ?? "";
    return message;
  },
};

const baseImportResourceResponse: object = { newId: "" };

export const ImportResourceResponse = {
  encode(
    message: ImportResourceResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.newId !== "") {
      writer.uint32(10).string(message.newId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ImportResourceResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseImportResourceResponse } as ImportResourceResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.newId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ImportResourceResponse {
    const message = { ...baseImportResourceResponse } as ImportResourceResponse;
    message.newId =
      object.newId !== undefined && object.newId !== null
        ? String(object.newId)
        : "";
    return message;
  },

  toJSON(message: ImportResourceResponse): unknown {
    const obj: any = {};
    message.newId !== undefined && (obj.newId = message.newId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ImportResourceResponse>, I>>(
    object: I
  ): ImportResourceResponse {
    const message = { ...baseImportResourceResponse } as ImportResourceResponse;
    message.newId = object.newId ?? "";
    return message;
  },
};

const baseDeleteResourceRequest: object = { id: "" };

export const DeleteResourceRequest = {
  encode(
    message: DeleteResourceRequest,
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
  ): DeleteResourceRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseDeleteResourceRequest } as DeleteResourceRequest;
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

  fromJSON(object: any): DeleteResourceRequest {
    const message = { ...baseDeleteResourceRequest } as DeleteResourceRequest;
    message.id =
      object.id !== undefined && object.id !== null ? String(object.id) : "";
    return message;
  },

  toJSON(message: DeleteResourceRequest): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<DeleteResourceRequest>, I>>(
    object: I
  ): DeleteResourceRequest {
    const message = { ...baseDeleteResourceRequest } as DeleteResourceRequest;
    message.id = object.id ?? "";
    return message;
  },
};

const baseDeleteResourceResponse: object = {};

export const DeleteResourceResponse = {
  encode(
    _: DeleteResourceResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): DeleteResourceResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseDeleteResourceResponse } as DeleteResourceResponse;
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

  fromJSON(_: any): DeleteResourceResponse {
    const message = { ...baseDeleteResourceResponse } as DeleteResourceResponse;
    return message;
  },

  toJSON(_: DeleteResourceResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<DeleteResourceResponse>, I>>(
    _: I
  ): DeleteResourceResponse {
    const message = { ...baseDeleteResourceResponse } as DeleteResourceResponse;
    return message;
  },
};

const baseRenameResourceRequest: object = { id: "", newPath: "" };

export const RenameResourceRequest = {
  encode(
    message: RenameResourceRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.newPath !== "") {
      writer.uint32(18).string(message.newPath);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): RenameResourceRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseRenameResourceRequest } as RenameResourceRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.newPath = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): RenameResourceRequest {
    const message = { ...baseRenameResourceRequest } as RenameResourceRequest;
    message.id =
      object.id !== undefined && object.id !== null ? String(object.id) : "";
    message.newPath =
      object.newPath !== undefined && object.newPath !== null
        ? String(object.newPath)
        : "";
    return message;
  },

  toJSON(message: RenameResourceRequest): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.newPath !== undefined && (obj.newPath = message.newPath);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<RenameResourceRequest>, I>>(
    object: I
  ): RenameResourceRequest {
    const message = { ...baseRenameResourceRequest } as RenameResourceRequest;
    message.id = object.id ?? "";
    message.newPath = object.newPath ?? "";
    return message;
  },
};

const baseRenameResourceResponse: object = {};

export const RenameResourceResponse = {
  encode(
    _: RenameResourceResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): RenameResourceResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseRenameResourceResponse } as RenameResourceResponse;
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

  fromJSON(_: any): RenameResourceResponse {
    const message = { ...baseRenameResourceResponse } as RenameResourceResponse;
    return message;
  },

  toJSON(_: RenameResourceResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<RenameResourceResponse>, I>>(
    _: I
  ): RenameResourceResponse {
    const message = { ...baseRenameResourceResponse } as RenameResourceResponse;
    return message;
  },
};

const baseCloneResourceRequest: object = { sourceId: "", clonePath: "" };

export const CloneResourceRequest = {
  encode(
    message: CloneResourceRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.sourceId !== "") {
      writer.uint32(10).string(message.sourceId);
    }
    if (message.clonePath !== "") {
      writer.uint32(18).string(message.clonePath);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CloneResourceRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCloneResourceRequest } as CloneResourceRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.sourceId = reader.string();
          break;
        case 2:
          message.clonePath = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CloneResourceRequest {
    const message = { ...baseCloneResourceRequest } as CloneResourceRequest;
    message.sourceId =
      object.sourceId !== undefined && object.sourceId !== null
        ? String(object.sourceId)
        : "";
    message.clonePath =
      object.clonePath !== undefined && object.clonePath !== null
        ? String(object.clonePath)
        : "";
    return message;
  },

  toJSON(message: CloneResourceRequest): unknown {
    const obj: any = {};
    message.sourceId !== undefined && (obj.sourceId = message.sourceId);
    message.clonePath !== undefined && (obj.clonePath = message.clonePath);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CloneResourceRequest>, I>>(
    object: I
  ): CloneResourceRequest {
    const message = { ...baseCloneResourceRequest } as CloneResourceRequest;
    message.sourceId = object.sourceId ?? "";
    message.clonePath = object.clonePath ?? "";
    return message;
  },
};

const baseCloneResourceResponse: object = { newId: "" };

export const CloneResourceResponse = {
  encode(
    message: CloneResourceResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.newId !== "") {
      writer.uint32(10).string(message.newId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CloneResourceResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCloneResourceResponse } as CloneResourceResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.newId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CloneResourceResponse {
    const message = { ...baseCloneResourceResponse } as CloneResourceResponse;
    message.newId =
      object.newId !== undefined && object.newId !== null
        ? String(object.newId)
        : "";
    return message;
  },

  toJSON(message: CloneResourceResponse): unknown {
    const obj: any = {};
    message.newId !== undefined && (obj.newId = message.newId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CloneResourceResponse>, I>>(
    object: I
  ): CloneResourceResponse {
    const message = { ...baseCloneResourceResponse } as CloneResourceResponse;
    message.newId = object.newId ?? "";
    return message;
  },
};

export interface ResourceBrowser {
  searchResources(
    request: DeepPartial<SearchResourcesRequest>,
    metadata?: grpc.Metadata
  ): Promise<SearchResourcesResponse>;
  createResource(
    request: DeepPartial<CreateResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<CreateResourceResponse>;
  getResourceTypeNames(
    request: DeepPartial<GetResourceTypeNamesRequest>,
    metadata?: grpc.Metadata
  ): Promise<GetResourceTypeNamesResponse>;
  importResource(
    request: DeepPartial<ImportResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<ImportResourceResponse>;
  deleteResource(
    request: DeepPartial<DeleteResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<DeleteResourceResponse>;
  renameResource(
    request: DeepPartial<RenameResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<RenameResourceResponse>;
  cloneResource(
    request: DeepPartial<CloneResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<CloneResourceResponse>;
}

export class ResourceBrowserClientImpl implements ResourceBrowser {
  private readonly rpc: Rpc;

  constructor(rpc: Rpc) {
    this.rpc = rpc;
    this.searchResources = this.searchResources.bind(this);
    this.createResource = this.createResource.bind(this);
    this.getResourceTypeNames = this.getResourceTypeNames.bind(this);
    this.importResource = this.importResource.bind(this);
    this.deleteResource = this.deleteResource.bind(this);
    this.renameResource = this.renameResource.bind(this);
    this.cloneResource = this.cloneResource.bind(this);
  }

  searchResources(
    request: DeepPartial<SearchResourcesRequest>,
    metadata?: grpc.Metadata
  ): Promise<SearchResourcesResponse> {
    return this.rpc.unary(
      ResourceBrowserSearchResourcesDesc,
      SearchResourcesRequest.fromPartial(request),
      metadata
    );
  }

  createResource(
    request: DeepPartial<CreateResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<CreateResourceResponse> {
    return this.rpc.unary(
      ResourceBrowserCreateResourceDesc,
      CreateResourceRequest.fromPartial(request),
      metadata
    );
  }

  getResourceTypeNames(
    request: DeepPartial<GetResourceTypeNamesRequest>,
    metadata?: grpc.Metadata
  ): Promise<GetResourceTypeNamesResponse> {
    return this.rpc.unary(
      ResourceBrowserGetResourceTypeNamesDesc,
      GetResourceTypeNamesRequest.fromPartial(request),
      metadata
    );
  }

  importResource(
    request: DeepPartial<ImportResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<ImportResourceResponse> {
    return this.rpc.unary(
      ResourceBrowserImportResourceDesc,
      ImportResourceRequest.fromPartial(request),
      metadata
    );
  }

  deleteResource(
    request: DeepPartial<DeleteResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<DeleteResourceResponse> {
    return this.rpc.unary(
      ResourceBrowserDeleteResourceDesc,
      DeleteResourceRequest.fromPartial(request),
      metadata
    );
  }

  renameResource(
    request: DeepPartial<RenameResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<RenameResourceResponse> {
    return this.rpc.unary(
      ResourceBrowserRenameResourceDesc,
      RenameResourceRequest.fromPartial(request),
      metadata
    );
  }

  cloneResource(
    request: DeepPartial<CloneResourceRequest>,
    metadata?: grpc.Metadata
  ): Promise<CloneResourceResponse> {
    return this.rpc.unary(
      ResourceBrowserCloneResourceDesc,
      CloneResourceRequest.fromPartial(request),
      metadata
    );
  }
}

export const ResourceBrowserDesc = {
  serviceName: "resource_browser.ResourceBrowser",
};

export const ResourceBrowserSearchResourcesDesc: UnaryMethodDefinitionish = {
  methodName: "SearchResources",
  service: ResourceBrowserDesc,
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

export const ResourceBrowserCreateResourceDesc: UnaryMethodDefinitionish = {
  methodName: "CreateResource",
  service: ResourceBrowserDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return CreateResourceRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...CreateResourceResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const ResourceBrowserGetResourceTypeNamesDesc: UnaryMethodDefinitionish =
  {
    methodName: "GetResourceTypeNames",
    service: ResourceBrowserDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return GetResourceTypeNamesRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...GetResourceTypeNamesResponse.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const ResourceBrowserImportResourceDesc: UnaryMethodDefinitionish = {
  methodName: "ImportResource",
  service: ResourceBrowserDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return ImportResourceRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...ImportResourceResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const ResourceBrowserDeleteResourceDesc: UnaryMethodDefinitionish = {
  methodName: "DeleteResource",
  service: ResourceBrowserDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return DeleteResourceRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...DeleteResourceResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const ResourceBrowserRenameResourceDesc: UnaryMethodDefinitionish = {
  methodName: "RenameResource",
  service: ResourceBrowserDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return RenameResourceRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...RenameResourceResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const ResourceBrowserCloneResourceDesc: UnaryMethodDefinitionish = {
  methodName: "CloneResource",
  service: ResourceBrowserDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return CloneResourceRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...CloneResourceResponse.decode(data),
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
