/* eslint-disable */
import Long from "long";
import _m0 from "protobufjs/minimal";

export const protobufPackage = "analytics";

export interface CallTreeNode {
  hash: number;
  beginMs: number;
  endMs: number;
  children: CallTreeNode[];
}

export interface ScopeDesc {
  name: string;
  filename: string;
  line: number;
  hash: number;
}

export interface CallTree {
  scopes: { [key: number]: ScopeDesc };
  root: CallTreeNode | undefined;
}

export interface CallTree_ScopesEntry {
  key: number;
  value: ScopeDesc | undefined;
}

function createBaseCallTreeNode(): CallTreeNode {
  return { hash: 0, beginMs: 0, endMs: 0, children: [] };
}

export const CallTreeNode = {
  encode(
    message: CallTreeNode,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.hash !== 0) {
      writer.uint32(8).uint32(message.hash);
    }
    if (message.beginMs !== 0) {
      writer.uint32(17).double(message.beginMs);
    }
    if (message.endMs !== 0) {
      writer.uint32(25).double(message.endMs);
    }
    for (const v of message.children) {
      CallTreeNode.encode(v!, writer.uint32(34).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): CallTreeNode {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseCallTreeNode();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.hash = reader.uint32();
          break;
        case 2:
          message.beginMs = reader.double();
          break;
        case 3:
          message.endMs = reader.double();
          break;
        case 4:
          message.children.push(CallTreeNode.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CallTreeNode {
    return {
      hash: isSet(object.hash) ? Number(object.hash) : 0,
      beginMs: isSet(object.beginMs) ? Number(object.beginMs) : 0,
      endMs: isSet(object.endMs) ? Number(object.endMs) : 0,
      children: Array.isArray(object?.children)
        ? object.children.map((e: any) => CallTreeNode.fromJSON(e))
        : [],
    };
  },

  toJSON(message: CallTreeNode): unknown {
    const obj: any = {};
    message.hash !== undefined && (obj.hash = Math.round(message.hash));
    message.beginMs !== undefined && (obj.beginMs = message.beginMs);
    message.endMs !== undefined && (obj.endMs = message.endMs);
    if (message.children) {
      obj.children = message.children.map((e) =>
        e ? CallTreeNode.toJSON(e) : undefined
      );
    } else {
      obj.children = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CallTreeNode>, I>>(
    object: I
  ): CallTreeNode {
    const message = createBaseCallTreeNode();
    message.hash = object.hash ?? 0;
    message.beginMs = object.beginMs ?? 0;
    message.endMs = object.endMs ?? 0;
    message.children =
      object.children?.map((e) => CallTreeNode.fromPartial(e)) || [];
    return message;
  },
};

function createBaseScopeDesc(): ScopeDesc {
  return { name: "", filename: "", line: 0, hash: 0 };
}

export const ScopeDesc = {
  encode(
    message: ScopeDesc,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.filename !== "") {
      writer.uint32(18).string(message.filename);
    }
    if (message.line !== 0) {
      writer.uint32(24).uint32(message.line);
    }
    if (message.hash !== 0) {
      writer.uint32(32).uint32(message.hash);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ScopeDesc {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseScopeDesc();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.filename = reader.string();
          break;
        case 3:
          message.line = reader.uint32();
          break;
        case 4:
          message.hash = reader.uint32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ScopeDesc {
    return {
      name: isSet(object.name) ? String(object.name) : "",
      filename: isSet(object.filename) ? String(object.filename) : "",
      line: isSet(object.line) ? Number(object.line) : 0,
      hash: isSet(object.hash) ? Number(object.hash) : 0,
    };
  },

  toJSON(message: ScopeDesc): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.filename !== undefined && (obj.filename = message.filename);
    message.line !== undefined && (obj.line = Math.round(message.line));
    message.hash !== undefined && (obj.hash = Math.round(message.hash));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ScopeDesc>, I>>(
    object: I
  ): ScopeDesc {
    const message = createBaseScopeDesc();
    message.name = object.name ?? "";
    message.filename = object.filename ?? "";
    message.line = object.line ?? 0;
    message.hash = object.hash ?? 0;
    return message;
  },
};

function createBaseCallTree(): CallTree {
  return { scopes: {}, root: undefined };
}

export const CallTree = {
  encode(
    message: CallTree,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    Object.entries(message.scopes).forEach(([key, value]) => {
      CallTree_ScopesEntry.encode(
        { key: key as any, value },
        writer.uint32(10).fork()
      ).ldelim();
    });
    if (message.root !== undefined) {
      CallTreeNode.encode(message.root, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): CallTree {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseCallTree();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          const entry1 = CallTree_ScopesEntry.decode(reader, reader.uint32());
          if (entry1.value !== undefined) {
            message.scopes[entry1.key] = entry1.value;
          }
          break;
        case 2:
          message.root = CallTreeNode.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CallTree {
    return {
      scopes: isObject(object.scopes)
        ? Object.entries(object.scopes).reduce<{ [key: number]: ScopeDesc }>(
            (acc, [key, value]) => {
              acc[Number(key)] = ScopeDesc.fromJSON(value);
              return acc;
            },
            {}
          )
        : {},
      root: isSet(object.root) ? CallTreeNode.fromJSON(object.root) : undefined,
    };
  },

  toJSON(message: CallTree): unknown {
    const obj: any = {};
    obj.scopes = {};
    if (message.scopes) {
      Object.entries(message.scopes).forEach(([k, v]) => {
        obj.scopes[k] = ScopeDesc.toJSON(v);
      });
    }
    message.root !== undefined &&
      (obj.root = message.root ? CallTreeNode.toJSON(message.root) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CallTree>, I>>(object: I): CallTree {
    const message = createBaseCallTree();
    message.scopes = Object.entries(object.scopes ?? {}).reduce<{
      [key: number]: ScopeDesc;
    }>((acc, [key, value]) => {
      if (value !== undefined) {
        acc[Number(key)] = ScopeDesc.fromPartial(value);
      }
      return acc;
    }, {});
    message.root =
      object.root !== undefined && object.root !== null
        ? CallTreeNode.fromPartial(object.root)
        : undefined;
    return message;
  },
};

function createBaseCallTree_ScopesEntry(): CallTree_ScopesEntry {
  return { key: 0, value: undefined };
}

export const CallTree_ScopesEntry = {
  encode(
    message: CallTree_ScopesEntry,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.key !== 0) {
      writer.uint32(8).uint32(message.key);
    }
    if (message.value !== undefined) {
      ScopeDesc.encode(message.value, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CallTree_ScopesEntry {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseCallTree_ScopesEntry();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.key = reader.uint32();
          break;
        case 2:
          message.value = ScopeDesc.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CallTree_ScopesEntry {
    return {
      key: isSet(object.key) ? Number(object.key) : 0,
      value: isSet(object.value) ? ScopeDesc.fromJSON(object.value) : undefined,
    };
  },

  toJSON(message: CallTree_ScopesEntry): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = Math.round(message.key));
    message.value !== undefined &&
      (obj.value = message.value ? ScopeDesc.toJSON(message.value) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CallTree_ScopesEntry>, I>>(
    object: I
  ): CallTree_ScopesEntry {
    const message = createBaseCallTree_ScopesEntry();
    message.key = object.key ?? 0;
    message.value =
      object.value !== undefined && object.value !== null
        ? ScopeDesc.fromPartial(object.value)
        : undefined;
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
