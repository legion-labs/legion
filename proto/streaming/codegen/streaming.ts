/* eslint-disable */
import Long from "long";
import { grpc } from "@improbable-eng/grpc-web";
import _m0 from "protobufjs/minimal";
import { BrowserHeaders } from "browser-headers";

export const protobufPackage = "streaming";

export interface InitializeStreamRequest {
  rtcSessionDescription: Uint8Array;
}

export interface InitializeStreamResponse {
  rtcSessionDescription: Uint8Array;
  error: string;
}

const baseInitializeStreamRequest: object = {};

export const InitializeStreamRequest = {
  encode(
    message: InitializeStreamRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.rtcSessionDescription.length !== 0) {
      writer.uint32(10).bytes(message.rtcSessionDescription);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): InitializeStreamRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseInitializeStreamRequest,
    } as InitializeStreamRequest;
    message.rtcSessionDescription = new Uint8Array();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.rtcSessionDescription = reader.bytes();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InitializeStreamRequest {
    const message = {
      ...baseInitializeStreamRequest,
    } as InitializeStreamRequest;
    message.rtcSessionDescription =
      object.rtcSessionDescription !== undefined &&
      object.rtcSessionDescription !== null
        ? bytesFromBase64(object.rtcSessionDescription)
        : new Uint8Array();
    return message;
  },

  toJSON(message: InitializeStreamRequest): unknown {
    const obj: any = {};
    message.rtcSessionDescription !== undefined &&
      (obj.rtcSessionDescription = base64FromBytes(
        message.rtcSessionDescription !== undefined
          ? message.rtcSessionDescription
          : new Uint8Array()
      ));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InitializeStreamRequest>, I>>(
    object: I
  ): InitializeStreamRequest {
    const message = {
      ...baseInitializeStreamRequest,
    } as InitializeStreamRequest;
    message.rtcSessionDescription =
      object.rtcSessionDescription ?? new Uint8Array();
    return message;
  },
};

const baseInitializeStreamResponse: object = { error: "" };

export const InitializeStreamResponse = {
  encode(
    message: InitializeStreamResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.rtcSessionDescription.length !== 0) {
      writer.uint32(10).bytes(message.rtcSessionDescription);
    }
    if (message.error !== "") {
      writer.uint32(18).string(message.error);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): InitializeStreamResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseInitializeStreamResponse,
    } as InitializeStreamResponse;
    message.rtcSessionDescription = new Uint8Array();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.rtcSessionDescription = reader.bytes();
          break;
        case 2:
          message.error = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InitializeStreamResponse {
    const message = {
      ...baseInitializeStreamResponse,
    } as InitializeStreamResponse;
    message.rtcSessionDescription =
      object.rtcSessionDescription !== undefined &&
      object.rtcSessionDescription !== null
        ? bytesFromBase64(object.rtcSessionDescription)
        : new Uint8Array();
    message.error =
      object.error !== undefined && object.error !== null
        ? String(object.error)
        : "";
    return message;
  },

  toJSON(message: InitializeStreamResponse): unknown {
    const obj: any = {};
    message.rtcSessionDescription !== undefined &&
      (obj.rtcSessionDescription = base64FromBytes(
        message.rtcSessionDescription !== undefined
          ? message.rtcSessionDescription
          : new Uint8Array()
      ));
    message.error !== undefined && (obj.error = message.error);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InitializeStreamResponse>, I>>(
    object: I
  ): InitializeStreamResponse {
    const message = {
      ...baseInitializeStreamResponse,
    } as InitializeStreamResponse;
    message.rtcSessionDescription =
      object.rtcSessionDescription ?? new Uint8Array();
    message.error = object.error ?? "";
    return message;
  },
};

export interface Streamer {
  initializeStream(
    request: DeepPartial<InitializeStreamRequest>,
    metadata?: grpc.Metadata
  ): Promise<InitializeStreamResponse>;
}

export class StreamerClientImpl implements Streamer {
  private readonly rpc: Rpc;

  constructor(rpc: Rpc) {
    this.rpc = rpc;
    this.initializeStream = this.initializeStream.bind(this);
  }

  initializeStream(
    request: DeepPartial<InitializeStreamRequest>,
    metadata?: grpc.Metadata
  ): Promise<InitializeStreamResponse> {
    return this.rpc.unary(
      StreamerInitializeStreamDesc,
      InitializeStreamRequest.fromPartial(request),
      metadata
    );
  }
}

export const StreamerDesc = {
  serviceName: "streaming.Streamer",
};

export const StreamerInitializeStreamDesc: UnaryMethodDefinitionish = {
  methodName: "InitializeStream",
  service: StreamerDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return InitializeStreamRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...InitializeStreamResponse.decode(data),
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

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}
