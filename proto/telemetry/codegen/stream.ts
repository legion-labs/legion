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

const baseUDTMember: object = {
  name: "",
  typeName: "",
  offset: 0,
  size: 0,
  isReference: false,
};

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
    const message = { ...baseUDTMember } as UDTMember;
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
    const message = { ...baseUDTMember } as UDTMember;
    message.name =
      object.name !== undefined && object.name !== null
        ? String(object.name)
        : "";
    message.typeName =
      object.typeName !== undefined && object.typeName !== null
        ? String(object.typeName)
        : "";
    message.offset =
      object.offset !== undefined && object.offset !== null
        ? Number(object.offset)
        : 0;
    message.size =
      object.size !== undefined && object.size !== null
        ? Number(object.size)
        : 0;
    message.isReference =
      object.isReference !== undefined && object.isReference !== null
        ? Boolean(object.isReference)
        : false;
    return message;
  },

  toJSON(message: UDTMember): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.typeName !== undefined && (obj.typeName = message.typeName);
    message.offset !== undefined && (obj.offset = message.offset);
    message.size !== undefined && (obj.size = message.size);
    message.isReference !== undefined &&
      (obj.isReference = message.isReference);
    return obj;
  },

  fromPartial(object: DeepPartial<UDTMember>): UDTMember {
    const message = { ...baseUDTMember } as UDTMember;
    message.name = object.name ?? "";
    message.typeName = object.typeName ?? "";
    message.offset = object.offset ?? 0;
    message.size = object.size ?? 0;
    message.isReference = object.isReference ?? false;
    return message;
  },
};

const baseUserDefinedType: object = { name: "", size: 0 };

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
    const message = { ...baseUserDefinedType } as UserDefinedType;
    message.members = [];
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
    const message = { ...baseUserDefinedType } as UserDefinedType;
    message.name =
      object.name !== undefined && object.name !== null
        ? String(object.name)
        : "";
    message.size =
      object.size !== undefined && object.size !== null
        ? Number(object.size)
        : 0;
    message.members = (object.members ?? []).map((e: any) =>
      UDTMember.fromJSON(e)
    );
    return message;
  },

  toJSON(message: UserDefinedType): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.size !== undefined && (obj.size = message.size);
    if (message.members) {
      obj.members = message.members.map((e) =>
        e ? UDTMember.toJSON(e) : undefined
      );
    } else {
      obj.members = [];
    }
    return obj;
  },

  fromPartial(object: DeepPartial<UserDefinedType>): UserDefinedType {
    const message = { ...baseUserDefinedType } as UserDefinedType;
    message.name = object.name ?? "";
    message.size = object.size ?? 0;
    message.members = (object.members ?? []).map((e) =>
      UDTMember.fromPartial(e)
    );
    return message;
  },
};

const baseContainerMetadata: object = {};

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
    const message = { ...baseContainerMetadata } as ContainerMetadata;
    message.types = [];
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
    const message = { ...baseContainerMetadata } as ContainerMetadata;
    message.types = (object.types ?? []).map((e: any) =>
      UserDefinedType.fromJSON(e)
    );
    return message;
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

  fromPartial(object: DeepPartial<ContainerMetadata>): ContainerMetadata {
    const message = { ...baseContainerMetadata } as ContainerMetadata;
    message.types = (object.types ?? []).map((e) =>
      UserDefinedType.fromPartial(e)
    );
    return message;
  },
};

const baseStream: object = { streamId: "", processId: "", tags: "" };

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
    const message = { ...baseStream } as Stream;
    message.tags = [];
    message.properties = {};
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
    const message = { ...baseStream } as Stream;
    message.streamId =
      object.streamId !== undefined && object.streamId !== null
        ? String(object.streamId)
        : "";
    message.processId =
      object.processId !== undefined && object.processId !== null
        ? String(object.processId)
        : "";
    message.dependenciesMetadata =
      object.dependenciesMetadata !== undefined &&
      object.dependenciesMetadata !== null
        ? ContainerMetadata.fromJSON(object.dependenciesMetadata)
        : undefined;
    message.objectsMetadata =
      object.objectsMetadata !== undefined && object.objectsMetadata !== null
        ? ContainerMetadata.fromJSON(object.objectsMetadata)
        : undefined;
    message.tags = (object.tags ?? []).map((e: any) => String(e));
    message.properties = Object.entries(object.properties ?? {}).reduce<{
      [key: string]: string;
    }>((acc, [key, value]) => {
      acc[key] = String(value);
      return acc;
    }, {});
    return message;
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

  fromPartial(object: DeepPartial<Stream>): Stream {
    const message = { ...baseStream } as Stream;
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
    message.tags = (object.tags ?? []).map((e) => e);
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

const baseStream_PropertiesEntry: object = { key: "", value: "" };

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
    const message = { ...baseStream_PropertiesEntry } as Stream_PropertiesEntry;
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
    const message = { ...baseStream_PropertiesEntry } as Stream_PropertiesEntry;
    message.key =
      object.key !== undefined && object.key !== null ? String(object.key) : "";
    message.value =
      object.value !== undefined && object.value !== null
        ? String(object.value)
        : "";
    return message;
  },

  toJSON(message: Stream_PropertiesEntry): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = message.key);
    message.value !== undefined && (obj.value = message.value);
    return obj;
  },

  fromPartial(
    object: DeepPartial<Stream_PropertiesEntry>
  ): Stream_PropertiesEntry {
    const message = { ...baseStream_PropertiesEntry } as Stream_PropertiesEntry;
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

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}
