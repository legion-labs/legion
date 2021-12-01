/* eslint-disable */
import Long from "long";
import _m0 from "protobufjs/minimal";

export const protobufPackage = "telemetry";

export interface Process {
  processId: string;
  exe: string;
  username: string;
  realname: string;
  computer: string;
  distro: string;
  cpuBrand: string;
  tscFrequency: number;
  /** RFC 3339 */
  startTime: string;
  startTicks: number;
  parentProcessId: string;
}

const baseProcess: object = {
  processId: "",
  exe: "",
  username: "",
  realname: "",
  computer: "",
  distro: "",
  cpuBrand: "",
  tscFrequency: 0,
  startTime: "",
  startTicks: 0,
  parentProcessId: "",
};

export const Process = {
  encode(
    message: Process,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.processId !== "") {
      writer.uint32(10).string(message.processId);
    }
    if (message.exe !== "") {
      writer.uint32(18).string(message.exe);
    }
    if (message.username !== "") {
      writer.uint32(26).string(message.username);
    }
    if (message.realname !== "") {
      writer.uint32(34).string(message.realname);
    }
    if (message.computer !== "") {
      writer.uint32(42).string(message.computer);
    }
    if (message.distro !== "") {
      writer.uint32(50).string(message.distro);
    }
    if (message.cpuBrand !== "") {
      writer.uint32(58).string(message.cpuBrand);
    }
    if (message.tscFrequency !== 0) {
      writer.uint32(64).uint64(message.tscFrequency);
    }
    if (message.startTime !== "") {
      writer.uint32(74).string(message.startTime);
    }
    if (message.startTicks !== 0) {
      writer.uint32(80).uint64(message.startTicks);
    }
    if (message.parentProcessId !== "") {
      writer.uint32(90).string(message.parentProcessId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Process {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseProcess } as Process;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.processId = reader.string();
          break;
        case 2:
          message.exe = reader.string();
          break;
        case 3:
          message.username = reader.string();
          break;
        case 4:
          message.realname = reader.string();
          break;
        case 5:
          message.computer = reader.string();
          break;
        case 6:
          message.distro = reader.string();
          break;
        case 7:
          message.cpuBrand = reader.string();
          break;
        case 8:
          message.tscFrequency = longToNumber(reader.uint64() as Long);
          break;
        case 9:
          message.startTime = reader.string();
          break;
        case 10:
          message.startTicks = longToNumber(reader.uint64() as Long);
          break;
        case 11:
          message.parentProcessId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Process {
    const message = { ...baseProcess } as Process;
    message.processId =
      object.processId !== undefined && object.processId !== null
        ? String(object.processId)
        : "";
    message.exe =
      object.exe !== undefined && object.exe !== null ? String(object.exe) : "";
    message.username =
      object.username !== undefined && object.username !== null
        ? String(object.username)
        : "";
    message.realname =
      object.realname !== undefined && object.realname !== null
        ? String(object.realname)
        : "";
    message.computer =
      object.computer !== undefined && object.computer !== null
        ? String(object.computer)
        : "";
    message.distro =
      object.distro !== undefined && object.distro !== null
        ? String(object.distro)
        : "";
    message.cpuBrand =
      object.cpuBrand !== undefined && object.cpuBrand !== null
        ? String(object.cpuBrand)
        : "";
    message.tscFrequency =
      object.tscFrequency !== undefined && object.tscFrequency !== null
        ? Number(object.tscFrequency)
        : 0;
    message.startTime =
      object.startTime !== undefined && object.startTime !== null
        ? String(object.startTime)
        : "";
    message.startTicks =
      object.startTicks !== undefined && object.startTicks !== null
        ? Number(object.startTicks)
        : 0;
    message.parentProcessId =
      object.parentProcessId !== undefined && object.parentProcessId !== null
        ? String(object.parentProcessId)
        : "";
    return message;
  },

  toJSON(message: Process): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    message.exe !== undefined && (obj.exe = message.exe);
    message.username !== undefined && (obj.username = message.username);
    message.realname !== undefined && (obj.realname = message.realname);
    message.computer !== undefined && (obj.computer = message.computer);
    message.distro !== undefined && (obj.distro = message.distro);
    message.cpuBrand !== undefined && (obj.cpuBrand = message.cpuBrand);
    message.tscFrequency !== undefined &&
      (obj.tscFrequency = message.tscFrequency);
    message.startTime !== undefined && (obj.startTime = message.startTime);
    message.startTicks !== undefined && (obj.startTicks = message.startTicks);
    message.parentProcessId !== undefined &&
      (obj.parentProcessId = message.parentProcessId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Process>, I>>(object: I): Process {
    const message = { ...baseProcess } as Process;
    message.processId = object.processId ?? "";
    message.exe = object.exe ?? "";
    message.username = object.username ?? "";
    message.realname = object.realname ?? "";
    message.computer = object.computer ?? "";
    message.distro = object.distro ?? "";
    message.cpuBrand = object.cpuBrand ?? "";
    message.tscFrequency = object.tscFrequency ?? 0;
    message.startTime = object.startTime ?? "";
    message.startTicks = object.startTicks ?? 0;
    message.parentProcessId = object.parentProcessId ?? "";
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
