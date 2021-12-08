/* eslint-disable */
import Long from "long";
import { grpc } from "@improbable-eng/grpc-web";
import _m0 from "protobufjs/minimal";
import { Observable } from "rxjs";
import { BrowserHeaders } from "browser-headers";
import { share } from "rxjs/operators";

export const protobufPackage = "streaming";

export interface InitializeStreamRequest {
  rtcSessionDescription: Uint8Array;
}

export interface InitializeStreamResponse {
  ok: InitializeStreamResponse_Ok | undefined;
  error: string | undefined;
}

export interface InitializeStreamResponse_Ok {
  rtcSessionDescription: Uint8Array;
  streamId: string;
}

export interface AddIceCandidatesRequest {
  streamId: string;
  iceCandidates: Uint8Array[];
}

export interface AddIceCandidatesResponse {
  ok: boolean;
}

export interface IceCandidateRequest {
  streamId: string;
}

export interface IceCandidateResponse {
  iceCandidate: Uint8Array;
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

const baseInitializeStreamResponse: object = {};

export const InitializeStreamResponse = {
  encode(
    message: InitializeStreamResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.ok !== undefined) {
      InitializeStreamResponse_Ok.encode(
        message.ok,
        writer.uint32(26).fork()
      ).ldelim();
    }
    if (message.error !== undefined) {
      writer.uint32(34).string(message.error);
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
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 3:
          message.ok = InitializeStreamResponse_Ok.decode(
            reader,
            reader.uint32()
          );
          break;
        case 4:
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
    message.ok =
      object.ok !== undefined && object.ok !== null
        ? InitializeStreamResponse_Ok.fromJSON(object.ok)
        : undefined;
    message.error =
      object.error !== undefined && object.error !== null
        ? String(object.error)
        : undefined;
    return message;
  },

  toJSON(message: InitializeStreamResponse): unknown {
    const obj: any = {};
    message.ok !== undefined &&
      (obj.ok = message.ok
        ? InitializeStreamResponse_Ok.toJSON(message.ok)
        : undefined);
    message.error !== undefined && (obj.error = message.error);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InitializeStreamResponse>, I>>(
    object: I
  ): InitializeStreamResponse {
    const message = {
      ...baseInitializeStreamResponse,
    } as InitializeStreamResponse;
    message.ok =
      object.ok !== undefined && object.ok !== null
        ? InitializeStreamResponse_Ok.fromPartial(object.ok)
        : undefined;
    message.error = object.error ?? undefined;
    return message;
  },
};

const baseInitializeStreamResponse_Ok: object = { streamId: "" };

export const InitializeStreamResponse_Ok = {
  encode(
    message: InitializeStreamResponse_Ok,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.rtcSessionDescription.length !== 0) {
      writer.uint32(10).bytes(message.rtcSessionDescription);
    }
    if (message.streamId !== "") {
      writer.uint32(18).string(message.streamId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): InitializeStreamResponse_Ok {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseInitializeStreamResponse_Ok,
    } as InitializeStreamResponse_Ok;
    message.rtcSessionDescription = new Uint8Array();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.rtcSessionDescription = reader.bytes();
          break;
        case 2:
          message.streamId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InitializeStreamResponse_Ok {
    const message = {
      ...baseInitializeStreamResponse_Ok,
    } as InitializeStreamResponse_Ok;
    message.rtcSessionDescription =
      object.rtcSessionDescription !== undefined &&
      object.rtcSessionDescription !== null
        ? bytesFromBase64(object.rtcSessionDescription)
        : new Uint8Array();
    message.streamId =
      object.streamId !== undefined && object.streamId !== null
        ? String(object.streamId)
        : "";
    return message;
  },

  toJSON(message: InitializeStreamResponse_Ok): unknown {
    const obj: any = {};
    message.rtcSessionDescription !== undefined &&
      (obj.rtcSessionDescription = base64FromBytes(
        message.rtcSessionDescription !== undefined
          ? message.rtcSessionDescription
          : new Uint8Array()
      ));
    message.streamId !== undefined && (obj.streamId = message.streamId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InitializeStreamResponse_Ok>, I>>(
    object: I
  ): InitializeStreamResponse_Ok {
    const message = {
      ...baseInitializeStreamResponse_Ok,
    } as InitializeStreamResponse_Ok;
    message.rtcSessionDescription =
      object.rtcSessionDescription ?? new Uint8Array();
    message.streamId = object.streamId ?? "";
    return message;
  },
};

const baseAddIceCandidatesRequest: object = { streamId: "" };

export const AddIceCandidatesRequest = {
  encode(
    message: AddIceCandidatesRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.streamId !== "") {
      writer.uint32(10).string(message.streamId);
    }
    for (const v of message.iceCandidates) {
      writer.uint32(18).bytes(v!);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): AddIceCandidatesRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseAddIceCandidatesRequest,
    } as AddIceCandidatesRequest;
    message.iceCandidates = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.streamId = reader.string();
          break;
        case 2:
          message.iceCandidates.push(reader.bytes());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): AddIceCandidatesRequest {
    const message = {
      ...baseAddIceCandidatesRequest,
    } as AddIceCandidatesRequest;
    message.streamId =
      object.streamId !== undefined && object.streamId !== null
        ? String(object.streamId)
        : "";
    message.iceCandidates = (object.iceCandidates ?? []).map((e: any) =>
      bytesFromBase64(e)
    );
    return message;
  },

  toJSON(message: AddIceCandidatesRequest): unknown {
    const obj: any = {};
    message.streamId !== undefined && (obj.streamId = message.streamId);
    if (message.iceCandidates) {
      obj.iceCandidates = message.iceCandidates.map((e) =>
        base64FromBytes(e !== undefined ? e : new Uint8Array())
      );
    } else {
      obj.iceCandidates = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<AddIceCandidatesRequest>, I>>(
    object: I
  ): AddIceCandidatesRequest {
    const message = {
      ...baseAddIceCandidatesRequest,
    } as AddIceCandidatesRequest;
    message.streamId = object.streamId ?? "";
    message.iceCandidates = object.iceCandidates?.map((e) => e) || [];
    return message;
  },
};

const baseAddIceCandidatesResponse: object = { ok: false };

export const AddIceCandidatesResponse = {
  encode(
    message: AddIceCandidatesResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.ok === true) {
      writer.uint32(8).bool(message.ok);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): AddIceCandidatesResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseAddIceCandidatesResponse,
    } as AddIceCandidatesResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.ok = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): AddIceCandidatesResponse {
    const message = {
      ...baseAddIceCandidatesResponse,
    } as AddIceCandidatesResponse;
    message.ok =
      object.ok !== undefined && object.ok !== null
        ? Boolean(object.ok)
        : false;
    return message;
  },

  toJSON(message: AddIceCandidatesResponse): unknown {
    const obj: any = {};
    message.ok !== undefined && (obj.ok = message.ok);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<AddIceCandidatesResponse>, I>>(
    object: I
  ): AddIceCandidatesResponse {
    const message = {
      ...baseAddIceCandidatesResponse,
    } as AddIceCandidatesResponse;
    message.ok = object.ok ?? false;
    return message;
  },
};

const baseIceCandidateRequest: object = { streamId: "" };

export const IceCandidateRequest = {
  encode(
    message: IceCandidateRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.streamId !== "") {
      writer.uint32(10).string(message.streamId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): IceCandidateRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseIceCandidateRequest } as IceCandidateRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.streamId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): IceCandidateRequest {
    const message = { ...baseIceCandidateRequest } as IceCandidateRequest;
    message.streamId =
      object.streamId !== undefined && object.streamId !== null
        ? String(object.streamId)
        : "";
    return message;
  },

  toJSON(message: IceCandidateRequest): unknown {
    const obj: any = {};
    message.streamId !== undefined && (obj.streamId = message.streamId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<IceCandidateRequest>, I>>(
    object: I
  ): IceCandidateRequest {
    const message = { ...baseIceCandidateRequest } as IceCandidateRequest;
    message.streamId = object.streamId ?? "";
    return message;
  },
};

const baseIceCandidateResponse: object = {};

export const IceCandidateResponse = {
  encode(
    message: IceCandidateResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.iceCandidate.length !== 0) {
      writer.uint32(10).bytes(message.iceCandidate);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): IceCandidateResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseIceCandidateResponse } as IceCandidateResponse;
    message.iceCandidate = new Uint8Array();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.iceCandidate = reader.bytes();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): IceCandidateResponse {
    const message = { ...baseIceCandidateResponse } as IceCandidateResponse;
    message.iceCandidate =
      object.iceCandidate !== undefined && object.iceCandidate !== null
        ? bytesFromBase64(object.iceCandidate)
        : new Uint8Array();
    return message;
  },

  toJSON(message: IceCandidateResponse): unknown {
    const obj: any = {};
    message.iceCandidate !== undefined &&
      (obj.iceCandidate = base64FromBytes(
        message.iceCandidate !== undefined
          ? message.iceCandidate
          : new Uint8Array()
      ));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<IceCandidateResponse>, I>>(
    object: I
  ): IceCandidateResponse {
    const message = { ...baseIceCandidateResponse } as IceCandidateResponse;
    message.iceCandidate = object.iceCandidate ?? new Uint8Array();
    return message;
  },
};

export interface Streamer {
  initializeStream(
    request: DeepPartial<InitializeStreamRequest>,
    metadata?: grpc.Metadata
  ): Promise<InitializeStreamResponse>;
  addIceCandidates(
    request: DeepPartial<AddIceCandidatesRequest>,
    metadata?: grpc.Metadata
  ): Promise<AddIceCandidatesResponse>;
  iceCandidates(
    request: DeepPartial<IceCandidateRequest>,
    metadata?: grpc.Metadata
  ): Observable<IceCandidateResponse>;
}

export class StreamerClientImpl implements Streamer {
  private readonly rpc: Rpc;

  constructor(rpc: Rpc) {
    this.rpc = rpc;
    this.initializeStream = this.initializeStream.bind(this);
    this.addIceCandidates = this.addIceCandidates.bind(this);
    this.iceCandidates = this.iceCandidates.bind(this);
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

  addIceCandidates(
    request: DeepPartial<AddIceCandidatesRequest>,
    metadata?: grpc.Metadata
  ): Promise<AddIceCandidatesResponse> {
    return this.rpc.unary(
      StreamerAddIceCandidatesDesc,
      AddIceCandidatesRequest.fromPartial(request),
      metadata
    );
  }

  iceCandidates(
    request: DeepPartial<IceCandidateRequest>,
    metadata?: grpc.Metadata
  ): Observable<IceCandidateResponse> {
    return this.rpc.invoke(
      StreamerIceCandidatesDesc,
      IceCandidateRequest.fromPartial(request),
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

export const StreamerAddIceCandidatesDesc: UnaryMethodDefinitionish = {
  methodName: "AddIceCandidates",
  service: StreamerDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return AddIceCandidatesRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...AddIceCandidatesResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const StreamerIceCandidatesDesc: UnaryMethodDefinitionish = {
  methodName: "IceCandidates",
  service: StreamerDesc,
  requestStream: false,
  responseStream: true,
  requestType: {
    serializeBinary() {
      return IceCandidateRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...IceCandidateResponse.decode(data),
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
  invoke<T extends UnaryMethodDefinitionish>(
    methodDesc: T,
    request: any,
    metadata: grpc.Metadata | undefined
  ): Observable<any>;
}

export class GrpcWebImpl {
  private host: string;
  private options: {
    transport?: grpc.TransportFactory;
    streamingTransport?: grpc.TransportFactory;
    debug?: boolean;
    metadata?: grpc.Metadata;
  };

  constructor(
    host: string,
    options: {
      transport?: grpc.TransportFactory;
      streamingTransport?: grpc.TransportFactory;
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

  invoke<T extends UnaryMethodDefinitionish>(
    methodDesc: T,
    _request: any,
    metadata: grpc.Metadata | undefined
  ): Observable<any> {
    // Status Response Codes (https://developers.google.com/maps-booking/reference/grpc-api/status_codes)
    const upStreamCodes = [2, 4, 8, 9, 10, 13, 14, 15];
    const DEFAULT_TIMEOUT_TIME: number = 3_000;
    const request = { ..._request, ...methodDesc.requestType };
    const maybeCombinedMetadata =
      metadata && this.options.metadata
        ? new BrowserHeaders({
            ...this.options?.metadata.headersMap,
            ...metadata?.headersMap,
          })
        : metadata || this.options.metadata;
    return new Observable((observer) => {
      const upStream = () => {
        const client = grpc.invoke(methodDesc, {
          host: this.host,
          request,
          transport: this.options.streamingTransport || this.options.transport,
          metadata: maybeCombinedMetadata,
          debug: this.options.debug,
          onMessage: (next) => observer.next(next),
          onEnd: (code: grpc.Code, message: string) => {
            if (code === 0) {
              observer.complete();
            } else if (upStreamCodes.includes(code)) {
              setTimeout(upStream, DEFAULT_TIMEOUT_TIME);
            } else {
              observer.error(new Error(`Error ${code} ${message}`));
            }
          },
        });
        observer.add(() => client.close());
      };
      upStream();
    }).pipe(share());
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
