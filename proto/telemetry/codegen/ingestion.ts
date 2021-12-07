/* eslint-disable */
import Long from "long";
import { grpc } from "@improbable-eng/grpc-web";
import _m0 from "protobufjs/minimal";
import { Process } from "./process";
import { Stream } from "./stream";
import { Block } from "./block";
import { BrowserHeaders } from "browser-headers";

export const protobufPackage = "ingestion";

export interface InsertReply {
  msg: string;
}

const baseInsertReply: object = { msg: "" };

export const InsertReply = {
  encode(
    message: InsertReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.msg !== "") {
      writer.uint32(10).string(message.msg);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): InsertReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInsertReply } as InsertReply;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.msg = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InsertReply {
    const message = { ...baseInsertReply } as InsertReply;
    message.msg =
      object.msg !== undefined && object.msg !== null ? String(object.msg) : "";
    return message;
  },

  toJSON(message: InsertReply): unknown {
    const obj: any = {};
    message.msg !== undefined && (obj.msg = message.msg);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertReply>, I>>(
    object: I
  ): InsertReply {
    const message = { ...baseInsertReply } as InsertReply;
    message.msg = object.msg ?? "";
    return message;
  },
};

export interface TelemetryIngestion {
  insert_process(
    request: DeepPartial<Process>,
    metadata?: grpc.Metadata
  ): Promise<InsertReply>;
  insert_stream(
    request: DeepPartial<Stream>,
    metadata?: grpc.Metadata
  ): Promise<InsertReply>;
  insert_block(
    request: DeepPartial<Block>,
    metadata?: grpc.Metadata
  ): Promise<InsertReply>;
}

export class TelemetryIngestionClientImpl implements TelemetryIngestion {
  private readonly rpc: Rpc;

  constructor(rpc: Rpc) {
    this.rpc = rpc;
    this.insert_process = this.insert_process.bind(this);
    this.insert_stream = this.insert_stream.bind(this);
    this.insert_block = this.insert_block.bind(this);
  }

  insert_process(
    request: DeepPartial<Process>,
    metadata?: grpc.Metadata
  ): Promise<InsertReply> {
    return this.rpc.unary(
      TelemetryIngestioninsert_processDesc,
      Process.fromPartial(request),
      metadata
    );
  }

  insert_stream(
    request: DeepPartial<Stream>,
    metadata?: grpc.Metadata
  ): Promise<InsertReply> {
    return this.rpc.unary(
      TelemetryIngestioninsert_streamDesc,
      Stream.fromPartial(request),
      metadata
    );
  }

  insert_block(
    request: DeepPartial<Block>,
    metadata?: grpc.Metadata
  ): Promise<InsertReply> {
    return this.rpc.unary(
      TelemetryIngestioninsert_blockDesc,
      Block.fromPartial(request),
      metadata
    );
  }
}

export const TelemetryIngestionDesc = {
  serviceName: "ingestion.TelemetryIngestion",
};

export const TelemetryIngestioninsert_processDesc: UnaryMethodDefinitionish = {
  methodName: "insert_process",
  service: TelemetryIngestionDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return Process.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...InsertReply.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const TelemetryIngestioninsert_streamDesc: UnaryMethodDefinitionish = {
  methodName: "insert_stream",
  service: TelemetryIngestionDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return Stream.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...InsertReply.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const TelemetryIngestioninsert_blockDesc: UnaryMethodDefinitionish = {
  methodName: "insert_block",
  service: TelemetryIngestionDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return Block.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...InsertReply.decode(data),
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
