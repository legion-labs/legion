/* eslint-disable */
import Long from "long";
import { grpc } from "@improbable-eng/grpc-web";
import _m0 from "protobufjs/minimal";
import { BrowserHeaders } from "browser-headers";
import { Process } from "./process";
import { Stream } from "./stream";
import { Block } from "./block";

export const protobufPackage = "analytics";

/** find_process */
export interface FindProcessRequest {
  processId: string;
}

export interface FindProcessReply {
  process: Process | undefined;
}

/** list_recent_processes */
export interface RecentProcessesRequest {}

export interface ProcessInstance {
  processInfo: Process | undefined;
  nbCpuBlocks: number;
  nbLogBlocks: number;
  nbMetricBlocks: number;
}

export interface ProcessListReply {
  processes: ProcessInstance[];
}

/** list_process_streams */
export interface ListProcessStreamsRequest {
  processId: string;
}

export interface ListStreamsReply {
  streams: Stream[];
}

/** list_stream_blocks */
export interface ListStreamBlocksRequest {
  streamId: string;
}

export interface ListStreamBlocksReply {
  blocks: Block[];
}

/**
 * block_spans
 * Span: represents a function call instance
 */
export interface Span {
  scopeHash: number;
  /** how many function calls are above this one in the thread */
  depth: number;
  beginMs: number;
  endMs: number;
}

export interface ScopeDesc {
  name: string;
  filename: string;
  line: number;
  hash: number;
}

export interface BlockSpansRequest {
  process: Process | undefined;
  stream: Stream | undefined;
  blockId: string;
}

export interface BlockSpansReply {
  scopes: ScopeDesc[];
  spans: Span[];
  blockId: string;
  beginMs: number;
  endMs: number;
  maxDepth: number;
}

/** process_cumulative_call_graph */
export interface ProcessCumulativeCallGraphRequest {
  process: Process | undefined;
  beginMs: number;
  endMs: number;
}

export interface NodeStats {
  sum: number;
  min: number;
  max: number;
  avg: number;
  median: number;
  count: number;
}

export interface CallGraphEdge {
  hash: number;
  weight: number;
}

export interface CumulativeCallGraphNode {
  hash: number;
  stats: NodeStats | undefined;
  callers: CallGraphEdge[];
  callees: CallGraphEdge[];
}

export interface CumulativeCallGraphReply {
  scopes: ScopeDesc[];
  nodes: CumulativeCallGraphNode[];
}

/** list_process_log_entries */
export interface ProcessLogRequest {
  process: Process | undefined;
}

export interface LogEntry {
  timeMs: number;
  msg: string;
}

export interface ProcessLogReply {
  entries: LogEntry[];
}

/** list_process_children */
export interface ListProcessChildrenRequest {
  processId: string;
}

export interface ProcessChildrenReply {
  processes: Process[];
}

const baseFindProcessRequest: object = { processId: "" };

export const FindProcessRequest = {
  encode(
    message: FindProcessRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.processId !== "") {
      writer.uint32(10).string(message.processId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FindProcessRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseFindProcessRequest } as FindProcessRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.processId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindProcessRequest {
    const message = { ...baseFindProcessRequest } as FindProcessRequest;
    message.processId =
      object.processId !== undefined && object.processId !== null
        ? String(object.processId)
        : "";
    return message;
  },

  toJSON(message: FindProcessRequest): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    return obj;
  },

  fromPartial(object: DeepPartial<FindProcessRequest>): FindProcessRequest {
    const message = { ...baseFindProcessRequest } as FindProcessRequest;
    message.processId = object.processId ?? "";
    return message;
  },
};

const baseFindProcessReply: object = {};

export const FindProcessReply = {
  encode(
    message: FindProcessReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.process !== undefined) {
      Process.encode(message.process, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FindProcessReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseFindProcessReply } as FindProcessReply;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.process = Process.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindProcessReply {
    const message = { ...baseFindProcessReply } as FindProcessReply;
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromJSON(object.process)
        : undefined;
    return message;
  },

  toJSON(message: FindProcessReply): unknown {
    const obj: any = {};
    message.process !== undefined &&
      (obj.process = message.process
        ? Process.toJSON(message.process)
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<FindProcessReply>): FindProcessReply {
    const message = { ...baseFindProcessReply } as FindProcessReply;
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromPartial(object.process)
        : undefined;
    return message;
  },
};

const baseRecentProcessesRequest: object = {};

export const RecentProcessesRequest = {
  encode(
    _: RecentProcessesRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): RecentProcessesRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseRecentProcessesRequest } as RecentProcessesRequest;
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

  fromJSON(_: any): RecentProcessesRequest {
    const message = { ...baseRecentProcessesRequest } as RecentProcessesRequest;
    return message;
  },

  toJSON(_: RecentProcessesRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial(_: DeepPartial<RecentProcessesRequest>): RecentProcessesRequest {
    const message = { ...baseRecentProcessesRequest } as RecentProcessesRequest;
    return message;
  },
};

const baseProcessInstance: object = {
  nbCpuBlocks: 0,
  nbLogBlocks: 0,
  nbMetricBlocks: 0,
};

export const ProcessInstance = {
  encode(
    message: ProcessInstance,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.processInfo !== undefined) {
      Process.encode(message.processInfo, writer.uint32(10).fork()).ldelim();
    }
    if (message.nbCpuBlocks !== 0) {
      writer.uint32(16).uint32(message.nbCpuBlocks);
    }
    if (message.nbLogBlocks !== 0) {
      writer.uint32(24).uint32(message.nbLogBlocks);
    }
    if (message.nbMetricBlocks !== 0) {
      writer.uint32(32).uint32(message.nbMetricBlocks);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ProcessInstance {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseProcessInstance } as ProcessInstance;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.processInfo = Process.decode(reader, reader.uint32());
          break;
        case 2:
          message.nbCpuBlocks = reader.uint32();
          break;
        case 3:
          message.nbLogBlocks = reader.uint32();
          break;
        case 4:
          message.nbMetricBlocks = reader.uint32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessInstance {
    const message = { ...baseProcessInstance } as ProcessInstance;
    message.processInfo =
      object.processInfo !== undefined && object.processInfo !== null
        ? Process.fromJSON(object.processInfo)
        : undefined;
    message.nbCpuBlocks =
      object.nbCpuBlocks !== undefined && object.nbCpuBlocks !== null
        ? Number(object.nbCpuBlocks)
        : 0;
    message.nbLogBlocks =
      object.nbLogBlocks !== undefined && object.nbLogBlocks !== null
        ? Number(object.nbLogBlocks)
        : 0;
    message.nbMetricBlocks =
      object.nbMetricBlocks !== undefined && object.nbMetricBlocks !== null
        ? Number(object.nbMetricBlocks)
        : 0;
    return message;
  },

  toJSON(message: ProcessInstance): unknown {
    const obj: any = {};
    message.processInfo !== undefined &&
      (obj.processInfo = message.processInfo
        ? Process.toJSON(message.processInfo)
        : undefined);
    message.nbCpuBlocks !== undefined &&
      (obj.nbCpuBlocks = message.nbCpuBlocks);
    message.nbLogBlocks !== undefined &&
      (obj.nbLogBlocks = message.nbLogBlocks);
    message.nbMetricBlocks !== undefined &&
      (obj.nbMetricBlocks = message.nbMetricBlocks);
    return obj;
  },

  fromPartial(object: DeepPartial<ProcessInstance>): ProcessInstance {
    const message = { ...baseProcessInstance } as ProcessInstance;
    message.processInfo =
      object.processInfo !== undefined && object.processInfo !== null
        ? Process.fromPartial(object.processInfo)
        : undefined;
    message.nbCpuBlocks = object.nbCpuBlocks ?? 0;
    message.nbLogBlocks = object.nbLogBlocks ?? 0;
    message.nbMetricBlocks = object.nbMetricBlocks ?? 0;
    return message;
  },
};

const baseProcessListReply: object = {};

export const ProcessListReply = {
  encode(
    message: ProcessListReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.processes) {
      ProcessInstance.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ProcessListReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseProcessListReply } as ProcessListReply;
    message.processes = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.processes.push(
            ProcessInstance.decode(reader, reader.uint32())
          );
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessListReply {
    const message = { ...baseProcessListReply } as ProcessListReply;
    message.processes = (object.processes ?? []).map((e: any) =>
      ProcessInstance.fromJSON(e)
    );
    return message;
  },

  toJSON(message: ProcessListReply): unknown {
    const obj: any = {};
    if (message.processes) {
      obj.processes = message.processes.map((e) =>
        e ? ProcessInstance.toJSON(e) : undefined
      );
    } else {
      obj.processes = [];
    }
    return obj;
  },

  fromPartial(object: DeepPartial<ProcessListReply>): ProcessListReply {
    const message = { ...baseProcessListReply } as ProcessListReply;
    message.processes = (object.processes ?? []).map((e) =>
      ProcessInstance.fromPartial(e)
    );
    return message;
  },
};

const baseListProcessStreamsRequest: object = { processId: "" };

export const ListProcessStreamsRequest = {
  encode(
    message: ListProcessStreamsRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.processId !== "") {
      writer.uint32(10).string(message.processId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ListProcessStreamsRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseListProcessStreamsRequest,
    } as ListProcessStreamsRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.processId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ListProcessStreamsRequest {
    const message = {
      ...baseListProcessStreamsRequest,
    } as ListProcessStreamsRequest;
    message.processId =
      object.processId !== undefined && object.processId !== null
        ? String(object.processId)
        : "";
    return message;
  },

  toJSON(message: ListProcessStreamsRequest): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    return obj;
  },

  fromPartial(
    object: DeepPartial<ListProcessStreamsRequest>
  ): ListProcessStreamsRequest {
    const message = {
      ...baseListProcessStreamsRequest,
    } as ListProcessStreamsRequest;
    message.processId = object.processId ?? "";
    return message;
  },
};

const baseListStreamsReply: object = {};

export const ListStreamsReply = {
  encode(
    message: ListStreamsReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.streams) {
      Stream.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ListStreamsReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseListStreamsReply } as ListStreamsReply;
    message.streams = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.streams.push(Stream.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ListStreamsReply {
    const message = { ...baseListStreamsReply } as ListStreamsReply;
    message.streams = (object.streams ?? []).map((e: any) =>
      Stream.fromJSON(e)
    );
    return message;
  },

  toJSON(message: ListStreamsReply): unknown {
    const obj: any = {};
    if (message.streams) {
      obj.streams = message.streams.map((e) =>
        e ? Stream.toJSON(e) : undefined
      );
    } else {
      obj.streams = [];
    }
    return obj;
  },

  fromPartial(object: DeepPartial<ListStreamsReply>): ListStreamsReply {
    const message = { ...baseListStreamsReply } as ListStreamsReply;
    message.streams = (object.streams ?? []).map((e) => Stream.fromPartial(e));
    return message;
  },
};

const baseListStreamBlocksRequest: object = { streamId: "" };

export const ListStreamBlocksRequest = {
  encode(
    message: ListStreamBlocksRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.streamId !== "") {
      writer.uint32(10).string(message.streamId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ListStreamBlocksRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseListStreamBlocksRequest,
    } as ListStreamBlocksRequest;
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

  fromJSON(object: any): ListStreamBlocksRequest {
    const message = {
      ...baseListStreamBlocksRequest,
    } as ListStreamBlocksRequest;
    message.streamId =
      object.streamId !== undefined && object.streamId !== null
        ? String(object.streamId)
        : "";
    return message;
  },

  toJSON(message: ListStreamBlocksRequest): unknown {
    const obj: any = {};
    message.streamId !== undefined && (obj.streamId = message.streamId);
    return obj;
  },

  fromPartial(
    object: DeepPartial<ListStreamBlocksRequest>
  ): ListStreamBlocksRequest {
    const message = {
      ...baseListStreamBlocksRequest,
    } as ListStreamBlocksRequest;
    message.streamId = object.streamId ?? "";
    return message;
  },
};

const baseListStreamBlocksReply: object = {};

export const ListStreamBlocksReply = {
  encode(
    message: ListStreamBlocksReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.blocks) {
      Block.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ListStreamBlocksReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseListStreamBlocksReply } as ListStreamBlocksReply;
    message.blocks = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.blocks.push(Block.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ListStreamBlocksReply {
    const message = { ...baseListStreamBlocksReply } as ListStreamBlocksReply;
    message.blocks = (object.blocks ?? []).map((e: any) => Block.fromJSON(e));
    return message;
  },

  toJSON(message: ListStreamBlocksReply): unknown {
    const obj: any = {};
    if (message.blocks) {
      obj.blocks = message.blocks.map((e) => (e ? Block.toJSON(e) : undefined));
    } else {
      obj.blocks = [];
    }
    return obj;
  },

  fromPartial(
    object: DeepPartial<ListStreamBlocksReply>
  ): ListStreamBlocksReply {
    const message = { ...baseListStreamBlocksReply } as ListStreamBlocksReply;
    message.blocks = (object.blocks ?? []).map((e) => Block.fromPartial(e));
    return message;
  },
};

const baseSpan: object = { scopeHash: 0, depth: 0, beginMs: 0, endMs: 0 };

export const Span = {
  encode(message: Span, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.scopeHash !== 0) {
      writer.uint32(8).uint32(message.scopeHash);
    }
    if (message.depth !== 0) {
      writer.uint32(16).uint32(message.depth);
    }
    if (message.beginMs !== 0) {
      writer.uint32(25).double(message.beginMs);
    }
    if (message.endMs !== 0) {
      writer.uint32(33).double(message.endMs);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Span {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseSpan } as Span;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.scopeHash = reader.uint32();
          break;
        case 2:
          message.depth = reader.uint32();
          break;
        case 3:
          message.beginMs = reader.double();
          break;
        case 4:
          message.endMs = reader.double();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Span {
    const message = { ...baseSpan } as Span;
    message.scopeHash =
      object.scopeHash !== undefined && object.scopeHash !== null
        ? Number(object.scopeHash)
        : 0;
    message.depth =
      object.depth !== undefined && object.depth !== null
        ? Number(object.depth)
        : 0;
    message.beginMs =
      object.beginMs !== undefined && object.beginMs !== null
        ? Number(object.beginMs)
        : 0;
    message.endMs =
      object.endMs !== undefined && object.endMs !== null
        ? Number(object.endMs)
        : 0;
    return message;
  },

  toJSON(message: Span): unknown {
    const obj: any = {};
    message.scopeHash !== undefined && (obj.scopeHash = message.scopeHash);
    message.depth !== undefined && (obj.depth = message.depth);
    message.beginMs !== undefined && (obj.beginMs = message.beginMs);
    message.endMs !== undefined && (obj.endMs = message.endMs);
    return obj;
  },

  fromPartial(object: DeepPartial<Span>): Span {
    const message = { ...baseSpan } as Span;
    message.scopeHash = object.scopeHash ?? 0;
    message.depth = object.depth ?? 0;
    message.beginMs = object.beginMs ?? 0;
    message.endMs = object.endMs ?? 0;
    return message;
  },
};

const baseScopeDesc: object = { name: "", filename: "", line: 0, hash: 0 };

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
    const message = { ...baseScopeDesc } as ScopeDesc;
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
    const message = { ...baseScopeDesc } as ScopeDesc;
    message.name =
      object.name !== undefined && object.name !== null
        ? String(object.name)
        : "";
    message.filename =
      object.filename !== undefined && object.filename !== null
        ? String(object.filename)
        : "";
    message.line =
      object.line !== undefined && object.line !== null
        ? Number(object.line)
        : 0;
    message.hash =
      object.hash !== undefined && object.hash !== null
        ? Number(object.hash)
        : 0;
    return message;
  },

  toJSON(message: ScopeDesc): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.filename !== undefined && (obj.filename = message.filename);
    message.line !== undefined && (obj.line = message.line);
    message.hash !== undefined && (obj.hash = message.hash);
    return obj;
  },

  fromPartial(object: DeepPartial<ScopeDesc>): ScopeDesc {
    const message = { ...baseScopeDesc } as ScopeDesc;
    message.name = object.name ?? "";
    message.filename = object.filename ?? "";
    message.line = object.line ?? 0;
    message.hash = object.hash ?? 0;
    return message;
  },
};

const baseBlockSpansRequest: object = { blockId: "" };

export const BlockSpansRequest = {
  encode(
    message: BlockSpansRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.process !== undefined) {
      Process.encode(message.process, writer.uint32(10).fork()).ldelim();
    }
    if (message.stream !== undefined) {
      Stream.encode(message.stream, writer.uint32(18).fork()).ldelim();
    }
    if (message.blockId !== "") {
      writer.uint32(26).string(message.blockId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BlockSpansRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseBlockSpansRequest } as BlockSpansRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.process = Process.decode(reader, reader.uint32());
          break;
        case 2:
          message.stream = Stream.decode(reader, reader.uint32());
          break;
        case 3:
          message.blockId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): BlockSpansRequest {
    const message = { ...baseBlockSpansRequest } as BlockSpansRequest;
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromJSON(object.process)
        : undefined;
    message.stream =
      object.stream !== undefined && object.stream !== null
        ? Stream.fromJSON(object.stream)
        : undefined;
    message.blockId =
      object.blockId !== undefined && object.blockId !== null
        ? String(object.blockId)
        : "";
    return message;
  },

  toJSON(message: BlockSpansRequest): unknown {
    const obj: any = {};
    message.process !== undefined &&
      (obj.process = message.process
        ? Process.toJSON(message.process)
        : undefined);
    message.stream !== undefined &&
      (obj.stream = message.stream ? Stream.toJSON(message.stream) : undefined);
    message.blockId !== undefined && (obj.blockId = message.blockId);
    return obj;
  },

  fromPartial(object: DeepPartial<BlockSpansRequest>): BlockSpansRequest {
    const message = { ...baseBlockSpansRequest } as BlockSpansRequest;
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromPartial(object.process)
        : undefined;
    message.stream =
      object.stream !== undefined && object.stream !== null
        ? Stream.fromPartial(object.stream)
        : undefined;
    message.blockId = object.blockId ?? "";
    return message;
  },
};

const baseBlockSpansReply: object = {
  blockId: "",
  beginMs: 0,
  endMs: 0,
  maxDepth: 0,
};

export const BlockSpansReply = {
  encode(
    message: BlockSpansReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.scopes) {
      ScopeDesc.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.spans) {
      Span.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    if (message.blockId !== "") {
      writer.uint32(26).string(message.blockId);
    }
    if (message.beginMs !== 0) {
      writer.uint32(33).double(message.beginMs);
    }
    if (message.endMs !== 0) {
      writer.uint32(41).double(message.endMs);
    }
    if (message.maxDepth !== 0) {
      writer.uint32(48).uint32(message.maxDepth);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BlockSpansReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseBlockSpansReply } as BlockSpansReply;
    message.scopes = [];
    message.spans = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.scopes.push(ScopeDesc.decode(reader, reader.uint32()));
          break;
        case 2:
          message.spans.push(Span.decode(reader, reader.uint32()));
          break;
        case 3:
          message.blockId = reader.string();
          break;
        case 4:
          message.beginMs = reader.double();
          break;
        case 5:
          message.endMs = reader.double();
          break;
        case 6:
          message.maxDepth = reader.uint32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): BlockSpansReply {
    const message = { ...baseBlockSpansReply } as BlockSpansReply;
    message.scopes = (object.scopes ?? []).map((e: any) =>
      ScopeDesc.fromJSON(e)
    );
    message.spans = (object.spans ?? []).map((e: any) => Span.fromJSON(e));
    message.blockId =
      object.blockId !== undefined && object.blockId !== null
        ? String(object.blockId)
        : "";
    message.beginMs =
      object.beginMs !== undefined && object.beginMs !== null
        ? Number(object.beginMs)
        : 0;
    message.endMs =
      object.endMs !== undefined && object.endMs !== null
        ? Number(object.endMs)
        : 0;
    message.maxDepth =
      object.maxDepth !== undefined && object.maxDepth !== null
        ? Number(object.maxDepth)
        : 0;
    return message;
  },

  toJSON(message: BlockSpansReply): unknown {
    const obj: any = {};
    if (message.scopes) {
      obj.scopes = message.scopes.map((e) =>
        e ? ScopeDesc.toJSON(e) : undefined
      );
    } else {
      obj.scopes = [];
    }
    if (message.spans) {
      obj.spans = message.spans.map((e) => (e ? Span.toJSON(e) : undefined));
    } else {
      obj.spans = [];
    }
    message.blockId !== undefined && (obj.blockId = message.blockId);
    message.beginMs !== undefined && (obj.beginMs = message.beginMs);
    message.endMs !== undefined && (obj.endMs = message.endMs);
    message.maxDepth !== undefined && (obj.maxDepth = message.maxDepth);
    return obj;
  },

  fromPartial(object: DeepPartial<BlockSpansReply>): BlockSpansReply {
    const message = { ...baseBlockSpansReply } as BlockSpansReply;
    message.scopes = (object.scopes ?? []).map((e) => ScopeDesc.fromPartial(e));
    message.spans = (object.spans ?? []).map((e) => Span.fromPartial(e));
    message.blockId = object.blockId ?? "";
    message.beginMs = object.beginMs ?? 0;
    message.endMs = object.endMs ?? 0;
    message.maxDepth = object.maxDepth ?? 0;
    return message;
  },
};

const baseProcessCumulativeCallGraphRequest: object = { beginMs: 0, endMs: 0 };

export const ProcessCumulativeCallGraphRequest = {
  encode(
    message: ProcessCumulativeCallGraphRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.process !== undefined) {
      Process.encode(message.process, writer.uint32(10).fork()).ldelim();
    }
    if (message.beginMs !== 0) {
      writer.uint32(17).double(message.beginMs);
    }
    if (message.endMs !== 0) {
      writer.uint32(25).double(message.endMs);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ProcessCumulativeCallGraphRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseProcessCumulativeCallGraphRequest,
    } as ProcessCumulativeCallGraphRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.process = Process.decode(reader, reader.uint32());
          break;
        case 2:
          message.beginMs = reader.double();
          break;
        case 3:
          message.endMs = reader.double();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessCumulativeCallGraphRequest {
    const message = {
      ...baseProcessCumulativeCallGraphRequest,
    } as ProcessCumulativeCallGraphRequest;
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromJSON(object.process)
        : undefined;
    message.beginMs =
      object.beginMs !== undefined && object.beginMs !== null
        ? Number(object.beginMs)
        : 0;
    message.endMs =
      object.endMs !== undefined && object.endMs !== null
        ? Number(object.endMs)
        : 0;
    return message;
  },

  toJSON(message: ProcessCumulativeCallGraphRequest): unknown {
    const obj: any = {};
    message.process !== undefined &&
      (obj.process = message.process
        ? Process.toJSON(message.process)
        : undefined);
    message.beginMs !== undefined && (obj.beginMs = message.beginMs);
    message.endMs !== undefined && (obj.endMs = message.endMs);
    return obj;
  },

  fromPartial(
    object: DeepPartial<ProcessCumulativeCallGraphRequest>
  ): ProcessCumulativeCallGraphRequest {
    const message = {
      ...baseProcessCumulativeCallGraphRequest,
    } as ProcessCumulativeCallGraphRequest;
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromPartial(object.process)
        : undefined;
    message.beginMs = object.beginMs ?? 0;
    message.endMs = object.endMs ?? 0;
    return message;
  },
};

const baseNodeStats: object = {
  sum: 0,
  min: 0,
  max: 0,
  avg: 0,
  median: 0,
  count: 0,
};

export const NodeStats = {
  encode(
    message: NodeStats,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.sum !== 0) {
      writer.uint32(9).double(message.sum);
    }
    if (message.min !== 0) {
      writer.uint32(17).double(message.min);
    }
    if (message.max !== 0) {
      writer.uint32(25).double(message.max);
    }
    if (message.avg !== 0) {
      writer.uint32(33).double(message.avg);
    }
    if (message.median !== 0) {
      writer.uint32(41).double(message.median);
    }
    if (message.count !== 0) {
      writer.uint32(48).uint64(message.count);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): NodeStats {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseNodeStats } as NodeStats;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.sum = reader.double();
          break;
        case 2:
          message.min = reader.double();
          break;
        case 3:
          message.max = reader.double();
          break;
        case 4:
          message.avg = reader.double();
          break;
        case 5:
          message.median = reader.double();
          break;
        case 6:
          message.count = longToNumber(reader.uint64() as Long);
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): NodeStats {
    const message = { ...baseNodeStats } as NodeStats;
    message.sum =
      object.sum !== undefined && object.sum !== null ? Number(object.sum) : 0;
    message.min =
      object.min !== undefined && object.min !== null ? Number(object.min) : 0;
    message.max =
      object.max !== undefined && object.max !== null ? Number(object.max) : 0;
    message.avg =
      object.avg !== undefined && object.avg !== null ? Number(object.avg) : 0;
    message.median =
      object.median !== undefined && object.median !== null
        ? Number(object.median)
        : 0;
    message.count =
      object.count !== undefined && object.count !== null
        ? Number(object.count)
        : 0;
    return message;
  },

  toJSON(message: NodeStats): unknown {
    const obj: any = {};
    message.sum !== undefined && (obj.sum = message.sum);
    message.min !== undefined && (obj.min = message.min);
    message.max !== undefined && (obj.max = message.max);
    message.avg !== undefined && (obj.avg = message.avg);
    message.median !== undefined && (obj.median = message.median);
    message.count !== undefined && (obj.count = message.count);
    return obj;
  },

  fromPartial(object: DeepPartial<NodeStats>): NodeStats {
    const message = { ...baseNodeStats } as NodeStats;
    message.sum = object.sum ?? 0;
    message.min = object.min ?? 0;
    message.max = object.max ?? 0;
    message.avg = object.avg ?? 0;
    message.median = object.median ?? 0;
    message.count = object.count ?? 0;
    return message;
  },
};

const baseCallGraphEdge: object = { hash: 0, weight: 0 };

export const CallGraphEdge = {
  encode(
    message: CallGraphEdge,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.hash !== 0) {
      writer.uint32(8).uint32(message.hash);
    }
    if (message.weight !== 0) {
      writer.uint32(17).double(message.weight);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): CallGraphEdge {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCallGraphEdge } as CallGraphEdge;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.hash = reader.uint32();
          break;
        case 2:
          message.weight = reader.double();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CallGraphEdge {
    const message = { ...baseCallGraphEdge } as CallGraphEdge;
    message.hash =
      object.hash !== undefined && object.hash !== null
        ? Number(object.hash)
        : 0;
    message.weight =
      object.weight !== undefined && object.weight !== null
        ? Number(object.weight)
        : 0;
    return message;
  },

  toJSON(message: CallGraphEdge): unknown {
    const obj: any = {};
    message.hash !== undefined && (obj.hash = message.hash);
    message.weight !== undefined && (obj.weight = message.weight);
    return obj;
  },

  fromPartial(object: DeepPartial<CallGraphEdge>): CallGraphEdge {
    const message = { ...baseCallGraphEdge } as CallGraphEdge;
    message.hash = object.hash ?? 0;
    message.weight = object.weight ?? 0;
    return message;
  },
};

const baseCumulativeCallGraphNode: object = { hash: 0 };

export const CumulativeCallGraphNode = {
  encode(
    message: CumulativeCallGraphNode,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.hash !== 0) {
      writer.uint32(8).uint32(message.hash);
    }
    if (message.stats !== undefined) {
      NodeStats.encode(message.stats, writer.uint32(18).fork()).ldelim();
    }
    for (const v of message.callers) {
      CallGraphEdge.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    for (const v of message.callees) {
      CallGraphEdge.encode(v!, writer.uint32(34).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CumulativeCallGraphNode {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseCumulativeCallGraphNode,
    } as CumulativeCallGraphNode;
    message.callers = [];
    message.callees = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.hash = reader.uint32();
          break;
        case 2:
          message.stats = NodeStats.decode(reader, reader.uint32());
          break;
        case 3:
          message.callers.push(CallGraphEdge.decode(reader, reader.uint32()));
          break;
        case 4:
          message.callees.push(CallGraphEdge.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CumulativeCallGraphNode {
    const message = {
      ...baseCumulativeCallGraphNode,
    } as CumulativeCallGraphNode;
    message.hash =
      object.hash !== undefined && object.hash !== null
        ? Number(object.hash)
        : 0;
    message.stats =
      object.stats !== undefined && object.stats !== null
        ? NodeStats.fromJSON(object.stats)
        : undefined;
    message.callers = (object.callers ?? []).map((e: any) =>
      CallGraphEdge.fromJSON(e)
    );
    message.callees = (object.callees ?? []).map((e: any) =>
      CallGraphEdge.fromJSON(e)
    );
    return message;
  },

  toJSON(message: CumulativeCallGraphNode): unknown {
    const obj: any = {};
    message.hash !== undefined && (obj.hash = message.hash);
    message.stats !== undefined &&
      (obj.stats = message.stats ? NodeStats.toJSON(message.stats) : undefined);
    if (message.callers) {
      obj.callers = message.callers.map((e) =>
        e ? CallGraphEdge.toJSON(e) : undefined
      );
    } else {
      obj.callers = [];
    }
    if (message.callees) {
      obj.callees = message.callees.map((e) =>
        e ? CallGraphEdge.toJSON(e) : undefined
      );
    } else {
      obj.callees = [];
    }
    return obj;
  },

  fromPartial(
    object: DeepPartial<CumulativeCallGraphNode>
  ): CumulativeCallGraphNode {
    const message = {
      ...baseCumulativeCallGraphNode,
    } as CumulativeCallGraphNode;
    message.hash = object.hash ?? 0;
    message.stats =
      object.stats !== undefined && object.stats !== null
        ? NodeStats.fromPartial(object.stats)
        : undefined;
    message.callers = (object.callers ?? []).map((e) =>
      CallGraphEdge.fromPartial(e)
    );
    message.callees = (object.callees ?? []).map((e) =>
      CallGraphEdge.fromPartial(e)
    );
    return message;
  },
};

const baseCumulativeCallGraphReply: object = {};

export const CumulativeCallGraphReply = {
  encode(
    message: CumulativeCallGraphReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.scopes) {
      ScopeDesc.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.nodes) {
      CumulativeCallGraphNode.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CumulativeCallGraphReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseCumulativeCallGraphReply,
    } as CumulativeCallGraphReply;
    message.scopes = [];
    message.nodes = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.scopes.push(ScopeDesc.decode(reader, reader.uint32()));
          break;
        case 2:
          message.nodes.push(
            CumulativeCallGraphNode.decode(reader, reader.uint32())
          );
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CumulativeCallGraphReply {
    const message = {
      ...baseCumulativeCallGraphReply,
    } as CumulativeCallGraphReply;
    message.scopes = (object.scopes ?? []).map((e: any) =>
      ScopeDesc.fromJSON(e)
    );
    message.nodes = (object.nodes ?? []).map((e: any) =>
      CumulativeCallGraphNode.fromJSON(e)
    );
    return message;
  },

  toJSON(message: CumulativeCallGraphReply): unknown {
    const obj: any = {};
    if (message.scopes) {
      obj.scopes = message.scopes.map((e) =>
        e ? ScopeDesc.toJSON(e) : undefined
      );
    } else {
      obj.scopes = [];
    }
    if (message.nodes) {
      obj.nodes = message.nodes.map((e) =>
        e ? CumulativeCallGraphNode.toJSON(e) : undefined
      );
    } else {
      obj.nodes = [];
    }
    return obj;
  },

  fromPartial(
    object: DeepPartial<CumulativeCallGraphReply>
  ): CumulativeCallGraphReply {
    const message = {
      ...baseCumulativeCallGraphReply,
    } as CumulativeCallGraphReply;
    message.scopes = (object.scopes ?? []).map((e) => ScopeDesc.fromPartial(e));
    message.nodes = (object.nodes ?? []).map((e) =>
      CumulativeCallGraphNode.fromPartial(e)
    );
    return message;
  },
};

const baseProcessLogRequest: object = {};

export const ProcessLogRequest = {
  encode(
    message: ProcessLogRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.process !== undefined) {
      Process.encode(message.process, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ProcessLogRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseProcessLogRequest } as ProcessLogRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.process = Process.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessLogRequest {
    const message = { ...baseProcessLogRequest } as ProcessLogRequest;
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromJSON(object.process)
        : undefined;
    return message;
  },

  toJSON(message: ProcessLogRequest): unknown {
    const obj: any = {};
    message.process !== undefined &&
      (obj.process = message.process
        ? Process.toJSON(message.process)
        : undefined);
    return obj;
  },

  fromPartial(object: DeepPartial<ProcessLogRequest>): ProcessLogRequest {
    const message = { ...baseProcessLogRequest } as ProcessLogRequest;
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromPartial(object.process)
        : undefined;
    return message;
  },
};

const baseLogEntry: object = { timeMs: 0, msg: "" };

export const LogEntry = {
  encode(
    message: LogEntry,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.timeMs !== 0) {
      writer.uint32(9).double(message.timeMs);
    }
    if (message.msg !== "") {
      writer.uint32(18).string(message.msg);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): LogEntry {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseLogEntry } as LogEntry;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.timeMs = reader.double();
          break;
        case 2:
          message.msg = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): LogEntry {
    const message = { ...baseLogEntry } as LogEntry;
    message.timeMs =
      object.timeMs !== undefined && object.timeMs !== null
        ? Number(object.timeMs)
        : 0;
    message.msg =
      object.msg !== undefined && object.msg !== null ? String(object.msg) : "";
    return message;
  },

  toJSON(message: LogEntry): unknown {
    const obj: any = {};
    message.timeMs !== undefined && (obj.timeMs = message.timeMs);
    message.msg !== undefined && (obj.msg = message.msg);
    return obj;
  },

  fromPartial(object: DeepPartial<LogEntry>): LogEntry {
    const message = { ...baseLogEntry } as LogEntry;
    message.timeMs = object.timeMs ?? 0;
    message.msg = object.msg ?? "";
    return message;
  },
};

const baseProcessLogReply: object = {};

export const ProcessLogReply = {
  encode(
    message: ProcessLogReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.entries) {
      LogEntry.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ProcessLogReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseProcessLogReply } as ProcessLogReply;
    message.entries = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.entries.push(LogEntry.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessLogReply {
    const message = { ...baseProcessLogReply } as ProcessLogReply;
    message.entries = (object.entries ?? []).map((e: any) =>
      LogEntry.fromJSON(e)
    );
    return message;
  },

  toJSON(message: ProcessLogReply): unknown {
    const obj: any = {};
    if (message.entries) {
      obj.entries = message.entries.map((e) =>
        e ? LogEntry.toJSON(e) : undefined
      );
    } else {
      obj.entries = [];
    }
    return obj;
  },

  fromPartial(object: DeepPartial<ProcessLogReply>): ProcessLogReply {
    const message = { ...baseProcessLogReply } as ProcessLogReply;
    message.entries = (object.entries ?? []).map((e) =>
      LogEntry.fromPartial(e)
    );
    return message;
  },
};

const baseListProcessChildrenRequest: object = { processId: "" };

export const ListProcessChildrenRequest = {
  encode(
    message: ListProcessChildrenRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.processId !== "") {
      writer.uint32(10).string(message.processId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ListProcessChildrenRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseListProcessChildrenRequest,
    } as ListProcessChildrenRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.processId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ListProcessChildrenRequest {
    const message = {
      ...baseListProcessChildrenRequest,
    } as ListProcessChildrenRequest;
    message.processId =
      object.processId !== undefined && object.processId !== null
        ? String(object.processId)
        : "";
    return message;
  },

  toJSON(message: ListProcessChildrenRequest): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    return obj;
  },

  fromPartial(
    object: DeepPartial<ListProcessChildrenRequest>
  ): ListProcessChildrenRequest {
    const message = {
      ...baseListProcessChildrenRequest,
    } as ListProcessChildrenRequest;
    message.processId = object.processId ?? "";
    return message;
  },
};

const baseProcessChildrenReply: object = {};

export const ProcessChildrenReply = {
  encode(
    message: ProcessChildrenReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.processes) {
      Process.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ProcessChildrenReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseProcessChildrenReply } as ProcessChildrenReply;
    message.processes = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.processes.push(Process.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessChildrenReply {
    const message = { ...baseProcessChildrenReply } as ProcessChildrenReply;
    message.processes = (object.processes ?? []).map((e: any) =>
      Process.fromJSON(e)
    );
    return message;
  },

  toJSON(message: ProcessChildrenReply): unknown {
    const obj: any = {};
    if (message.processes) {
      obj.processes = message.processes.map((e) =>
        e ? Process.toJSON(e) : undefined
      );
    } else {
      obj.processes = [];
    }
    return obj;
  },

  fromPartial(object: DeepPartial<ProcessChildrenReply>): ProcessChildrenReply {
    const message = { ...baseProcessChildrenReply } as ProcessChildrenReply;
    message.processes = (object.processes ?? []).map((e) =>
      Process.fromPartial(e)
    );
    return message;
  },
};

export interface PerformanceAnalytics {
  block_spans(
    request: DeepPartial<BlockSpansRequest>,
    metadata?: grpc.Metadata
  ): Promise<BlockSpansReply>;
  process_cumulative_call_graph(
    request: DeepPartial<ProcessCumulativeCallGraphRequest>,
    metadata?: grpc.Metadata
  ): Promise<CumulativeCallGraphReply>;
  find_process(
    request: DeepPartial<FindProcessRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindProcessReply>;
  list_process_children(
    request: DeepPartial<ListProcessChildrenRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessChildrenReply>;
  list_process_log_entries(
    request: DeepPartial<ProcessLogRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessLogReply>;
  list_process_streams(
    request: DeepPartial<ListProcessStreamsRequest>,
    metadata?: grpc.Metadata
  ): Promise<ListStreamsReply>;
  list_recent_processes(
    request: DeepPartial<RecentProcessesRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessListReply>;
  list_stream_blocks(
    request: DeepPartial<ListStreamBlocksRequest>,
    metadata?: grpc.Metadata
  ): Promise<ListStreamBlocksReply>;
}

export class PerformanceAnalyticsClientImpl implements PerformanceAnalytics {
  private readonly rpc: Rpc;

  constructor(rpc: Rpc) {
    this.rpc = rpc;
    this.block_spans = this.block_spans.bind(this);
    this.process_cumulative_call_graph =
      this.process_cumulative_call_graph.bind(this);
    this.find_process = this.find_process.bind(this);
    this.list_process_children = this.list_process_children.bind(this);
    this.list_process_log_entries = this.list_process_log_entries.bind(this);
    this.list_process_streams = this.list_process_streams.bind(this);
    this.list_recent_processes = this.list_recent_processes.bind(this);
    this.list_stream_blocks = this.list_stream_blocks.bind(this);
  }

  block_spans(
    request: DeepPartial<BlockSpansRequest>,
    metadata?: grpc.Metadata
  ): Promise<BlockSpansReply> {
    return this.rpc.unary(
      PerformanceAnalyticsblock_spansDesc,
      BlockSpansRequest.fromPartial(request),
      metadata
    );
  }

  process_cumulative_call_graph(
    request: DeepPartial<ProcessCumulativeCallGraphRequest>,
    metadata?: grpc.Metadata
  ): Promise<CumulativeCallGraphReply> {
    return this.rpc.unary(
      PerformanceAnalyticsprocess_cumulative_call_graphDesc,
      ProcessCumulativeCallGraphRequest.fromPartial(request),
      metadata
    );
  }

  find_process(
    request: DeepPartial<FindProcessRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindProcessReply> {
    return this.rpc.unary(
      PerformanceAnalyticsfind_processDesc,
      FindProcessRequest.fromPartial(request),
      metadata
    );
  }

  list_process_children(
    request: DeepPartial<ListProcessChildrenRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessChildrenReply> {
    return this.rpc.unary(
      PerformanceAnalyticslist_process_childrenDesc,
      ListProcessChildrenRequest.fromPartial(request),
      metadata
    );
  }

  list_process_log_entries(
    request: DeepPartial<ProcessLogRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessLogReply> {
    return this.rpc.unary(
      PerformanceAnalyticslist_process_log_entriesDesc,
      ProcessLogRequest.fromPartial(request),
      metadata
    );
  }

  list_process_streams(
    request: DeepPartial<ListProcessStreamsRequest>,
    metadata?: grpc.Metadata
  ): Promise<ListStreamsReply> {
    return this.rpc.unary(
      PerformanceAnalyticslist_process_streamsDesc,
      ListProcessStreamsRequest.fromPartial(request),
      metadata
    );
  }

  list_recent_processes(
    request: DeepPartial<RecentProcessesRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessListReply> {
    return this.rpc.unary(
      PerformanceAnalyticslist_recent_processesDesc,
      RecentProcessesRequest.fromPartial(request),
      metadata
    );
  }

  list_stream_blocks(
    request: DeepPartial<ListStreamBlocksRequest>,
    metadata?: grpc.Metadata
  ): Promise<ListStreamBlocksReply> {
    return this.rpc.unary(
      PerformanceAnalyticslist_stream_blocksDesc,
      ListStreamBlocksRequest.fromPartial(request),
      metadata
    );
  }
}

export const PerformanceAnalyticsDesc = {
  serviceName: "analytics.PerformanceAnalytics",
};

export const PerformanceAnalyticsblock_spansDesc: UnaryMethodDefinitionish = {
  methodName: "block_spans",
  service: PerformanceAnalyticsDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return BlockSpansRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...BlockSpansReply.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const PerformanceAnalyticsprocess_cumulative_call_graphDesc: UnaryMethodDefinitionish =
  {
    methodName: "process_cumulative_call_graph",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return ProcessCumulativeCallGraphRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...CumulativeCallGraphReply.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PerformanceAnalyticsfind_processDesc: UnaryMethodDefinitionish = {
  methodName: "find_process",
  service: PerformanceAnalyticsDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return FindProcessRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...FindProcessReply.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const PerformanceAnalyticslist_process_childrenDesc: UnaryMethodDefinitionish =
  {
    methodName: "list_process_children",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return ListProcessChildrenRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...ProcessChildrenReply.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PerformanceAnalyticslist_process_log_entriesDesc: UnaryMethodDefinitionish =
  {
    methodName: "list_process_log_entries",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return ProcessLogRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...ProcessLogReply.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PerformanceAnalyticslist_process_streamsDesc: UnaryMethodDefinitionish =
  {
    methodName: "list_process_streams",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return ListProcessStreamsRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...ListStreamsReply.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PerformanceAnalyticslist_recent_processesDesc: UnaryMethodDefinitionish =
  {
    methodName: "list_recent_processes",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return RecentProcessesRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...ProcessListReply.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PerformanceAnalyticslist_stream_blocksDesc: UnaryMethodDefinitionish =
  {
    methodName: "list_stream_blocks",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return ListStreamBlocksRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...ListStreamBlocksReply.decode(data),
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
