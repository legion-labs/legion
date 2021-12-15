/* eslint-disable */
import Long from "long";
import _m0 from "protobufjs/minimal";

export const protobufPackage = "telemetry";

export interface BlockPayload {
  dependencies: Uint8Array;
  objects: Uint8Array;
}

export interface Block {
  blockId: string;
  streamId: string;
  /**
   * we send both RFC3339 times and ticks to be able to calibrate the tick
   * frequency
   */
  beginTime: string;
  beginTicks: number;
  endTime: string;
  endTicks: number;
  payload: BlockPayload | undefined;
  nbObjects: number;
}

const baseBlockPayload: object = {};

export const BlockPayload = {
  encode(
    message: BlockPayload,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.dependencies.length !== 0) {
      writer.uint32(10).bytes(message.dependencies);
    }
    if (message.objects.length !== 0) {
      writer.uint32(18).bytes(message.objects);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BlockPayload {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseBlockPayload } as BlockPayload;
    message.dependencies = new Uint8Array();
    message.objects = new Uint8Array();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.dependencies = reader.bytes();
          break;
        case 2:
          message.objects = reader.bytes();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): BlockPayload {
    const message = { ...baseBlockPayload } as BlockPayload;
    message.dependencies =
      object.dependencies !== undefined && object.dependencies !== null
        ? bytesFromBase64(object.dependencies)
        : new Uint8Array();
    message.objects =
      object.objects !== undefined && object.objects !== null
        ? bytesFromBase64(object.objects)
        : new Uint8Array();
    return message;
  },

  toJSON(message: BlockPayload): unknown {
    const obj: any = {};
    message.dependencies !== undefined &&
      (obj.dependencies = base64FromBytes(
        message.dependencies !== undefined
          ? message.dependencies
          : new Uint8Array()
      ));
    message.objects !== undefined &&
      (obj.objects = base64FromBytes(
        message.objects !== undefined ? message.objects : new Uint8Array()
      ));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<BlockPayload>, I>>(
    object: I
  ): BlockPayload {
    const message = { ...baseBlockPayload } as BlockPayload;
    message.dependencies = object.dependencies ?? new Uint8Array();
    message.objects = object.objects ?? new Uint8Array();
    return message;
  },
};

const baseBlock: object = {
  blockId: "",
  streamId: "",
  beginTime: "",
  beginTicks: 0,
  endTime: "",
  endTicks: 0,
  nbObjects: 0,
};

export const Block = {
  encode(message: Block, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.blockId !== "") {
      writer.uint32(10).string(message.blockId);
    }
    if (message.streamId !== "") {
      writer.uint32(18).string(message.streamId);
    }
    if (message.beginTime !== "") {
      writer.uint32(26).string(message.beginTime);
    }
    if (message.beginTicks !== 0) {
      writer.uint32(32).int64(message.beginTicks);
    }
    if (message.endTime !== "") {
      writer.uint32(42).string(message.endTime);
    }
    if (message.endTicks !== 0) {
      writer.uint32(48).int64(message.endTicks);
    }
    if (message.payload !== undefined) {
      BlockPayload.encode(message.payload, writer.uint32(58).fork()).ldelim();
    }
    if (message.nbObjects !== 0) {
      writer.uint32(64).int32(message.nbObjects);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Block {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseBlock } as Block;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.blockId = reader.string();
          break;
        case 2:
          message.streamId = reader.string();
          break;
        case 3:
          message.beginTime = reader.string();
          break;
        case 4:
          message.beginTicks = longToNumber(reader.int64() as Long);
          break;
        case 5:
          message.endTime = reader.string();
          break;
        case 6:
          message.endTicks = longToNumber(reader.int64() as Long);
          break;
        case 7:
          message.payload = BlockPayload.decode(reader, reader.uint32());
          break;
        case 8:
          message.nbObjects = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Block {
    const message = { ...baseBlock } as Block;
    message.blockId =
      object.blockId !== undefined && object.blockId !== null
        ? String(object.blockId)
        : "";
    message.streamId =
      object.streamId !== undefined && object.streamId !== null
        ? String(object.streamId)
        : "";
    message.beginTime =
      object.beginTime !== undefined && object.beginTime !== null
        ? String(object.beginTime)
        : "";
    message.beginTicks =
      object.beginTicks !== undefined && object.beginTicks !== null
        ? Number(object.beginTicks)
        : 0;
    message.endTime =
      object.endTime !== undefined && object.endTime !== null
        ? String(object.endTime)
        : "";
    message.endTicks =
      object.endTicks !== undefined && object.endTicks !== null
        ? Number(object.endTicks)
        : 0;
    message.payload =
      object.payload !== undefined && object.payload !== null
        ? BlockPayload.fromJSON(object.payload)
        : undefined;
    message.nbObjects =
      object.nbObjects !== undefined && object.nbObjects !== null
        ? Number(object.nbObjects)
        : 0;
    return message;
  },

  toJSON(message: Block): unknown {
    const obj: any = {};
    message.blockId !== undefined && (obj.blockId = message.blockId);
    message.streamId !== undefined && (obj.streamId = message.streamId);
    message.beginTime !== undefined && (obj.beginTime = message.beginTime);
    message.beginTicks !== undefined &&
      (obj.beginTicks = Math.round(message.beginTicks));
    message.endTime !== undefined && (obj.endTime = message.endTime);
    message.endTicks !== undefined &&
      (obj.endTicks = Math.round(message.endTicks));
    message.payload !== undefined &&
      (obj.payload = message.payload
        ? BlockPayload.toJSON(message.payload)
        : undefined);
    message.nbObjects !== undefined &&
      (obj.nbObjects = Math.round(message.nbObjects));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Block>, I>>(object: I): Block {
    const message = { ...baseBlock } as Block;
    message.blockId = object.blockId ?? "";
    message.streamId = object.streamId ?? "";
    message.beginTime = object.beginTime ?? "";
    message.beginTicks = object.beginTicks ?? 0;
    message.endTime = object.endTime ?? "";
    message.endTicks = object.endTicks ?? 0;
    message.payload =
      object.payload !== undefined && object.payload !== null
        ? BlockPayload.fromPartial(object.payload)
        : undefined;
    message.nbObjects = object.nbObjects ?? 0;
    return message;
  },
};

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
