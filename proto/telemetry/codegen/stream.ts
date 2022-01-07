/* eslint-disable */
import Long from "long";
import _m0 from "protobufjs/minimal";

export const protobufPackage = "telemetry";

export interface UDTMember {
  name: string;
  typeName: string;
  offset: number;
  size: number;
  isReference: boolean;
}

export interface UserDefinedType {
  name: string;
  size: number;
  members: UDTMember[];
}

export interface ContainerMetadata {
  types: UserDefinedType[];
}

export interface Stream {
  streamId: string;
  processId: string;
  dependenciesMetadata: ContainerMetadata | undefined;
  objectsMetadata: ContainerMetadata | undefined;
  tags: string[];
  properties: { [key: string]: string };
}

export interface Stream_PropertiesEntry {
  key: string;
  value: string;
}

function createBaseUDTMember(): UDTMember {
  return { name: "", typeName: "", offset: 0, size: 0, isReference: false };
}

export const UDTMember = {
  encode(
    message: UDTMember,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.typeName !== "") {
      writer.uint32(18).string(message.typeName);
    }
    if (message.offset !== 0) {
      writer.uint32(24).uint32(message.offset);
    }
    if (message.size !== 0) {
      writer.uint32(32).uint32(message.size);
    }
    if (message.isReference === true) {
      writer.uint32(40).bool(message.isReference);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UDTMember {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseUDTMember();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.typeName = reader.string();
          break;
        case 3:
          message.offset = reader.uint32();
          break;
        case 4:
          message.size = reader.uint32();
          break;
        case 5:
          message.isReference = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): UDTMember {
    return {
      name: isSet(object.name) ? String(object.name) : "",
      typeName: isSet(object.typeName) ? String(object.typeName) : "",
      offset: isSet(object.offset) ? Number(object.offset) : 0,
      size: isSet(object.size) ? Number(object.size) : 0,
      isReference: isSet(object.isReference)
        ? Boolean(object.isReference)
        : false,
    };
  },

  toJSON(message: UDTMember): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.typeName !== undefined && (obj.typeName = message.typeName);
    message.offset !== undefined && (obj.offset = Math.round(message.offset));
    message.size !== undefined && (obj.size = Math.round(message.size));
    message.isReference !== undefined &&
      (obj.isReference = message.isReference);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<UDTMember>, I>>(
    object: I
  ): UDTMember {
    const message = createBaseUDTMember();
    message.name = object.name ?? "";
    message.typeName = object.typeName ?? "";
    message.offset = object.offset ?? 0;
    message.size = object.size ?? 0;
    message.isReference = object.isReference ?? false;
    return message;
  },
};

function createBaseUserDefinedType(): UserDefinedType {
  return { name: "", size: 0, members: [] };
}

export const UserDefinedType = {
  encode(
    message: UserDefinedType,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.size !== 0) {
      writer.uint32(16).uint32(message.size);
    }
    for (const v of message.members) {
      UDTMember.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UserDefinedType {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseUserDefinedType();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.size = reader.uint32();
          break;
        case 3:
          message.members.push(UDTMember.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): UserDefinedType {
    return {
      name: isSet(object.name) ? String(object.name) : "",
      size: isSet(object.size) ? Number(object.size) : 0,
      members: Array.isArray(object?.members)
        ? object.members.map((e: any) => UDTMember.fromJSON(e))
        : [],
    };
  },

  toJSON(message: UserDefinedType): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.size !== undefined && (obj.size = Math.round(message.size));
    if (message.members) {
      obj.members = message.members.map((e) =>
        e ? UDTMember.toJSON(e) : undefined
      );
    } else {
      obj.members = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<UserDefinedType>, I>>(
    object: I
  ): UserDefinedType {
    const message = createBaseUserDefinedType();
    message.name = object.name ?? "";
    message.size = object.size ?? 0;
    message.members =
      object.members?.map((e) => UDTMember.fromPartial(e)) || [];
    return message;
  },
};

function createBaseContainerMetadata(): ContainerMetadata {
  return { types: [] };
}

export const ContainerMetadata = {
  encode(
    message: ContainerMetadata,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.types) {
      UserDefinedType.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ContainerMetadata {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseContainerMetadata();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.types.push(UserDefinedType.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ContainerMetadata {
    return {
      types: Array.isArray(object?.types)
        ? object.types.map((e: any) => UserDefinedType.fromJSON(e))
        : [],
    };
  },

  toJSON(message: ContainerMetadata): unknown {
    const obj: any = {};
    if (message.types) {
      obj.types = message.types.map((e) =>
        e ? UserDefinedType.toJSON(e) : undefined
      );
    } else {
      obj.types = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ContainerMetadata>, I>>(
    object: I
  ): ContainerMetadata {
    const message = createBaseContainerMetadata();
    message.types =
      object.types?.map((e) => UserDefinedType.fromPartial(e)) || [];
    return message;
  },
};

function createBaseStream(): Stream {
  return {
    streamId: "",
    processId: "",
    dependenciesMetadata: undefined,
    objectsMetadata: undefined,
    tags: [],
    properties: {},
  };
}

export const Stream = {
  encode(
    message: Stream,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.streamId !== "") {
      writer.uint32(10).string(message.streamId);
    }
    if (message.processId !== "") {
      writer.uint32(18).string(message.processId);
    }
    if (message.dependenciesMetadata !== undefined) {
      ContainerMetadata.encode(
        message.dependenciesMetadata,
        writer.uint32(26).fork()
      ).ldelim();
    }
    if (message.objectsMetadata !== undefined) {
      ContainerMetadata.encode(
        message.objectsMetadata,
        writer.uint32(34).fork()
      ).ldelim();
    }
    for (const v of message.tags) {
      writer.uint32(42).string(v!);
    }
    Object.entries(message.properties).forEach(([key, value]) => {
      Stream_PropertiesEntry.encode(
        { key: key as any, value },
        writer.uint32(50).fork()
      ).ldelim();
    });
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Stream {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseStream();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.streamId = reader.string();
          break;
        case 2:
          message.processId = reader.string();
          break;
        case 3:
          message.dependenciesMetadata = ContainerMetadata.decode(
            reader,
            reader.uint32()
          );
          break;
        case 4:
          message.objectsMetadata = ContainerMetadata.decode(
            reader,
            reader.uint32()
          );
          break;
        case 5:
          message.tags.push(reader.string());
          break;
        case 6:
          const entry6 = Stream_PropertiesEntry.decode(reader, reader.uint32());
          if (entry6.value !== undefined) {
            message.properties[entry6.key] = entry6.value;
          }
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Stream {
    return {
      streamId: isSet(object.streamId) ? String(object.streamId) : "",
      processId: isSet(object.processId) ? String(object.processId) : "",
      dependenciesMetadata: isSet(object.dependenciesMetadata)
        ? ContainerMetadata.fromJSON(object.dependenciesMetadata)
        : undefined,
      objectsMetadata: isSet(object.objectsMetadata)
        ? ContainerMetadata.fromJSON(object.objectsMetadata)
        : undefined,
      tags: Array.isArray(object?.tags)
        ? object.tags.map((e: any) => String(e))
        : [],
      properties: isObject(object.properties)
        ? Object.entries(object.properties).reduce<{ [key: string]: string }>(
            (acc, [key, value]) => {
              acc[key] = String(value);
              return acc;
            },
            {}
          )
        : {},
    };
  },

  toJSON(message: Stream): unknown {
    const obj: any = {};
    message.streamId !== undefined && (obj.streamId = message.streamId);
    message.processId !== undefined && (obj.processId = message.processId);
    message.dependenciesMetadata !== undefined &&
      (obj.dependenciesMetadata = message.dependenciesMetadata
        ? ContainerMetadata.toJSON(message.dependenciesMetadata)
        : undefined);
    message.objectsMetadata !== undefined &&
      (obj.objectsMetadata = message.objectsMetadata
        ? ContainerMetadata.toJSON(message.objectsMetadata)
        : undefined);
    if (message.tags) {
      obj.tags = message.tags.map((e) => e);
    } else {
      obj.tags = [];
    }
    obj.properties = {};
    if (message.properties) {
      Object.entries(message.properties).forEach(([k, v]) => {
        obj.properties[k] = v;
      });
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Stream>, I>>(object: I): Stream {
    const message = createBaseStream();
    message.streamId = object.streamId ?? "";
    message.processId = object.processId ?? "";
    message.dependenciesMetadata =
      object.dependenciesMetadata !== undefined &&
      object.dependenciesMetadata !== null
        ? ContainerMetadata.fromPartial(object.dependenciesMetadata)
        : undefined;
    message.objectsMetadata =
      object.objectsMetadata !== undefined && object.objectsMetadata !== null
        ? ContainerMetadata.fromPartial(object.objectsMetadata)
        : undefined;
    message.tags = object.tags?.map((e) => e) || [];
    message.properties = Object.entries(object.properties ?? {}).reduce<{
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

function createBaseStream_PropertiesEntry(): Stream_PropertiesEntry {
  return { key: "", value: "" };
}

export const Stream_PropertiesEntry = {
  encode(
    message: Stream_PropertiesEntry,
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
  ): Stream_PropertiesEntry {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseStream_PropertiesEntry();
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

  fromJSON(object: any): Stream_PropertiesEntry {
    return {
      key: isSet(object.key) ? String(object.key) : "",
      value: isSet(object.value) ? String(object.value) : "",
    };
  },

  toJSON(message: Stream_PropertiesEntry): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = message.key);
    message.value !== undefined && (obj.value = message.value);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Stream_PropertiesEntry>, I>>(
    object: I
  ): Stream_PropertiesEntry {
    const message = createBaseStream_PropertiesEntry();
    message.key = object.key ?? "";
    message.value = object.value ?? "";
    return message;
  },
};

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

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}

function isObject(value: any): boolean {
  return typeof value === "object" && value !== null;
}

function isSet(value: any): boolean {
  return value !== null && value !== undefined;
}
