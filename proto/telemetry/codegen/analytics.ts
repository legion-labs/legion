/* eslint-disable */
import Long from "long";
import { grpc } from "@improbable-eng/grpc-web";
import _m0 from "protobufjs/minimal";
import { BrowserHeaders } from "browser-headers";
import { Process } from "./process";
import { Stream } from "./stream";
import { Block } from "./block";
import { ScopeDesc } from "./calltree";

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

/** search_processes */
export interface SearchProcessRequest {
  search: string;
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
  beginMs: number;
  endMs: number;
  /** [0-255] non-linear transformation of occupancy for spans that are a lower level of detail */
  alpha: number;
}

export interface BlockSpansRequest {
  process: Process | undefined;
  stream: Stream | undefined;
  blockId: string;
  lodId: number;
}

/** one span track contains spans at one height of call stack */
export interface SpanTrack {
  spans: Span[];
}

export interface SpanBlockLOD {
  lodId: number;
  tracks: SpanTrack[];
}

export interface BlockSpansReply {
  scopes: { [key: number]: ScopeDesc };
  lod: SpanBlockLOD | undefined;
  blockId: string;
  beginMs: number;
  endMs: number;
}

export interface BlockSpansReply_ScopesEntry {
  key: number;
  value: ScopeDesc | undefined;
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
  scopes: { [key: number]: ScopeDesc };
  nodes: CumulativeCallGraphNode[];
}

export interface CumulativeCallGraphReply_ScopesEntry {
  key: number;
  value: ScopeDesc | undefined;
}

/** list_process_log_entries */
export interface ProcessLogRequest {
  process: Process | undefined;
  /** included */
  begin: number;
  /** excluded */
  end: number;
}

export interface LogEntry {
  timeMs: number;
  msg: string;
}

export interface ProcessLogReply {
  entries: LogEntry[];
  /** included */
  begin: number;
  /** excluded */
  end: number;
}

/** nb_process_log_entries(ProcessNbLogEntriesRequest) returns (ProcessNbLogEntriesReply); */
export interface ProcessNbLogEntriesRequest {
  processId: string;
}

export interface ProcessNbLogEntriesReply {
  count: number;
}

/** list_process_children */
export interface ListProcessChildrenRequest {
  processId: string;
}

export interface ProcessChildrenReply {
  processes: Process[];
}

/** list_process_metrics */
export interface ListProcessMetricsRequest {
  processId: string;
}

export interface MetricDesc {
  name: string;
  unit: string;
}

export interface ProcessMetricsReply {
  metrics: MetricDesc[];
  minTimeMs: number;
  maxTimeMs: number;
}

/** fetch_process_metric(FetchProcessMetricRequest) returns (ProcessMetricReply); */
export interface FetchProcessMetricRequest {
  processId: string;
  metricName: string;
  unit: string;
  beginMs: number;
  endMs: number;
}

export interface MetricDataPoint {
  timeMs: number;
  value: number;
}

export interface ProcessMetricReply {
  points: MetricDataPoint[];
}

function createBaseFindProcessRequest(): FindProcessRequest {
  return { processId: "" };
}

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
    const message = createBaseFindProcessRequest();
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
    return {
      processId: isSet(object.processId) ? String(object.processId) : "",
    };
  },

  toJSON(message: FindProcessRequest): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FindProcessRequest>, I>>(
    object: I
  ): FindProcessRequest {
    const message = createBaseFindProcessRequest();
    message.processId = object.processId ?? "";
    return message;
  },
};

function createBaseFindProcessReply(): FindProcessReply {
  return { process: undefined };
}

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
    const message = createBaseFindProcessReply();
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
    return {
      process: isSet(object.process)
        ? Process.fromJSON(object.process)
        : undefined,
    };
  },

  toJSON(message: FindProcessReply): unknown {
    const obj: any = {};
    message.process !== undefined &&
      (obj.process = message.process
        ? Process.toJSON(message.process)
        : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FindProcessReply>, I>>(
    object: I
  ): FindProcessReply {
    const message = createBaseFindProcessReply();
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromPartial(object.process)
        : undefined;
    return message;
  },
};

function createBaseRecentProcessesRequest(): RecentProcessesRequest {
  return {};
}

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
    const message = createBaseRecentProcessesRequest();
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
    return {};
  },

  toJSON(_: RecentProcessesRequest): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<RecentProcessesRequest>, I>>(
    _: I
  ): RecentProcessesRequest {
    const message = createBaseRecentProcessesRequest();
    return message;
  },
};

function createBaseProcessInstance(): ProcessInstance {
  return {
    processInfo: undefined,
    nbCpuBlocks: 0,
    nbLogBlocks: 0,
    nbMetricBlocks: 0,
  };
}

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
    const message = createBaseProcessInstance();
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
    return {
      processInfo: isSet(object.processInfo)
        ? Process.fromJSON(object.processInfo)
        : undefined,
      nbCpuBlocks: isSet(object.nbCpuBlocks) ? Number(object.nbCpuBlocks) : 0,
      nbLogBlocks: isSet(object.nbLogBlocks) ? Number(object.nbLogBlocks) : 0,
      nbMetricBlocks: isSet(object.nbMetricBlocks)
        ? Number(object.nbMetricBlocks)
        : 0,
    };
  },

  toJSON(message: ProcessInstance): unknown {
    const obj: any = {};
    message.processInfo !== undefined &&
      (obj.processInfo = message.processInfo
        ? Process.toJSON(message.processInfo)
        : undefined);
    message.nbCpuBlocks !== undefined &&
      (obj.nbCpuBlocks = Math.round(message.nbCpuBlocks));
    message.nbLogBlocks !== undefined &&
      (obj.nbLogBlocks = Math.round(message.nbLogBlocks));
    message.nbMetricBlocks !== undefined &&
      (obj.nbMetricBlocks = Math.round(message.nbMetricBlocks));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ProcessInstance>, I>>(
    object: I
  ): ProcessInstance {
    const message = createBaseProcessInstance();
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

function createBaseProcessListReply(): ProcessListReply {
  return { processes: [] };
}

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
    const message = createBaseProcessListReply();
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
    return {
      processes: Array.isArray(object?.processes)
        ? object.processes.map((e: any) => ProcessInstance.fromJSON(e))
        : [],
    };
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

  fromPartial<I extends Exact<DeepPartial<ProcessListReply>, I>>(
    object: I
  ): ProcessListReply {
    const message = createBaseProcessListReply();
    message.processes =
      object.processes?.map((e) => ProcessInstance.fromPartial(e)) || [];
    return message;
  },
};

function createBaseSearchProcessRequest(): SearchProcessRequest {
  return { search: "" };
}

export const SearchProcessRequest = {
  encode(
    message: SearchProcessRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.search !== "") {
      writer.uint32(10).string(message.search);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): SearchProcessRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseSearchProcessRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.search = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): SearchProcessRequest {
    return {
      search: isSet(object.search) ? String(object.search) : "",
    };
  },

  toJSON(message: SearchProcessRequest): unknown {
    const obj: any = {};
    message.search !== undefined && (obj.search = message.search);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<SearchProcessRequest>, I>>(
    object: I
  ): SearchProcessRequest {
    const message = createBaseSearchProcessRequest();
    message.search = object.search ?? "";
    return message;
  },
};

function createBaseListProcessStreamsRequest(): ListProcessStreamsRequest {
  return { processId: "" };
}

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
    const message = createBaseListProcessStreamsRequest();
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
    return {
      processId: isSet(object.processId) ? String(object.processId) : "",
    };
  },

  toJSON(message: ListProcessStreamsRequest): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ListProcessStreamsRequest>, I>>(
    object: I
  ): ListProcessStreamsRequest {
    const message = createBaseListProcessStreamsRequest();
    message.processId = object.processId ?? "";
    return message;
  },
};

function createBaseListStreamsReply(): ListStreamsReply {
  return { streams: [] };
}

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
    const message = createBaseListStreamsReply();
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
    return {
      streams: Array.isArray(object?.streams)
        ? object.streams.map((e: any) => Stream.fromJSON(e))
        : [],
    };
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

  fromPartial<I extends Exact<DeepPartial<ListStreamsReply>, I>>(
    object: I
  ): ListStreamsReply {
    const message = createBaseListStreamsReply();
    message.streams = object.streams?.map((e) => Stream.fromPartial(e)) || [];
    return message;
  },
};

function createBaseListStreamBlocksRequest(): ListStreamBlocksRequest {
  return { streamId: "" };
}

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
    const message = createBaseListStreamBlocksRequest();
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
    return {
      streamId: isSet(object.streamId) ? String(object.streamId) : "",
    };
  },

  toJSON(message: ListStreamBlocksRequest): unknown {
    const obj: any = {};
    message.streamId !== undefined && (obj.streamId = message.streamId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ListStreamBlocksRequest>, I>>(
    object: I
  ): ListStreamBlocksRequest {
    const message = createBaseListStreamBlocksRequest();
    message.streamId = object.streamId ?? "";
    return message;
  },
};

function createBaseListStreamBlocksReply(): ListStreamBlocksReply {
  return { blocks: [] };
}

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
    const message = createBaseListStreamBlocksReply();
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
    return {
      blocks: Array.isArray(object?.blocks)
        ? object.blocks.map((e: any) => Block.fromJSON(e))
        : [],
    };
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

  fromPartial<I extends Exact<DeepPartial<ListStreamBlocksReply>, I>>(
    object: I
  ): ListStreamBlocksReply {
    const message = createBaseListStreamBlocksReply();
    message.blocks = object.blocks?.map((e) => Block.fromPartial(e)) || [];
    return message;
  },
};

function createBaseSpan(): Span {
  return { scopeHash: 0, beginMs: 0, endMs: 0, alpha: 0 };
}

export const Span = {
  encode(message: Span, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.scopeHash !== 0) {
      writer.uint32(8).uint32(message.scopeHash);
    }
    if (message.beginMs !== 0) {
      writer.uint32(17).double(message.beginMs);
    }
    if (message.endMs !== 0) {
      writer.uint32(25).double(message.endMs);
    }
    if (message.alpha !== 0) {
      writer.uint32(32).uint32(message.alpha);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Span {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseSpan();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.scopeHash = reader.uint32();
          break;
        case 2:
          message.beginMs = reader.double();
          break;
        case 3:
          message.endMs = reader.double();
          break;
        case 4:
          message.alpha = reader.uint32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Span {
    return {
      scopeHash: isSet(object.scopeHash) ? Number(object.scopeHash) : 0,
      beginMs: isSet(object.beginMs) ? Number(object.beginMs) : 0,
      endMs: isSet(object.endMs) ? Number(object.endMs) : 0,
      alpha: isSet(object.alpha) ? Number(object.alpha) : 0,
    };
  },

  toJSON(message: Span): unknown {
    const obj: any = {};
    message.scopeHash !== undefined &&
      (obj.scopeHash = Math.round(message.scopeHash));
    message.beginMs !== undefined && (obj.beginMs = message.beginMs);
    message.endMs !== undefined && (obj.endMs = message.endMs);
    message.alpha !== undefined && (obj.alpha = Math.round(message.alpha));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Span>, I>>(object: I): Span {
    const message = createBaseSpan();
    message.scopeHash = object.scopeHash ?? 0;
    message.beginMs = object.beginMs ?? 0;
    message.endMs = object.endMs ?? 0;
    message.alpha = object.alpha ?? 0;
    return message;
  },
};

function createBaseBlockSpansRequest(): BlockSpansRequest {
  return { process: undefined, stream: undefined, blockId: "", lodId: 0 };
}

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
    if (message.lodId !== 0) {
      writer.uint32(32).uint32(message.lodId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BlockSpansRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseBlockSpansRequest();
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
        case 4:
          message.lodId = reader.uint32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): BlockSpansRequest {
    return {
      process: isSet(object.process)
        ? Process.fromJSON(object.process)
        : undefined,
      stream: isSet(object.stream) ? Stream.fromJSON(object.stream) : undefined,
      blockId: isSet(object.blockId) ? String(object.blockId) : "",
      lodId: isSet(object.lodId) ? Number(object.lodId) : 0,
    };
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
    message.lodId !== undefined && (obj.lodId = Math.round(message.lodId));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<BlockSpansRequest>, I>>(
    object: I
  ): BlockSpansRequest {
    const message = createBaseBlockSpansRequest();
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromPartial(object.process)
        : undefined;
    message.stream =
      object.stream !== undefined && object.stream !== null
        ? Stream.fromPartial(object.stream)
        : undefined;
    message.blockId = object.blockId ?? "";
    message.lodId = object.lodId ?? 0;
    return message;
  },
};

function createBaseSpanTrack(): SpanTrack {
  return { spans: [] };
}

export const SpanTrack = {
  encode(
    message: SpanTrack,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.spans) {
      Span.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SpanTrack {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseSpanTrack();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.spans.push(Span.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): SpanTrack {
    return {
      spans: Array.isArray(object?.spans)
        ? object.spans.map((e: any) => Span.fromJSON(e))
        : [],
    };
  },

  toJSON(message: SpanTrack): unknown {
    const obj: any = {};
    if (message.spans) {
      obj.spans = message.spans.map((e) => (e ? Span.toJSON(e) : undefined));
    } else {
      obj.spans = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<SpanTrack>, I>>(
    object: I
  ): SpanTrack {
    const message = createBaseSpanTrack();
    message.spans = object.spans?.map((e) => Span.fromPartial(e)) || [];
    return message;
  },
};

function createBaseSpanBlockLOD(): SpanBlockLOD {
  return { lodId: 0, tracks: [] };
}

export const SpanBlockLOD = {
  encode(
    message: SpanBlockLOD,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.lodId !== 0) {
      writer.uint32(8).uint32(message.lodId);
    }
    for (const v of message.tracks) {
      SpanTrack.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SpanBlockLOD {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseSpanBlockLOD();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.lodId = reader.uint32();
          break;
        case 2:
          message.tracks.push(SpanTrack.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): SpanBlockLOD {
    return {
      lodId: isSet(object.lodId) ? Number(object.lodId) : 0,
      tracks: Array.isArray(object?.tracks)
        ? object.tracks.map((e: any) => SpanTrack.fromJSON(e))
        : [],
    };
  },

  toJSON(message: SpanBlockLOD): unknown {
    const obj: any = {};
    message.lodId !== undefined && (obj.lodId = Math.round(message.lodId));
    if (message.tracks) {
      obj.tracks = message.tracks.map((e) =>
        e ? SpanTrack.toJSON(e) : undefined
      );
    } else {
      obj.tracks = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<SpanBlockLOD>, I>>(
    object: I
  ): SpanBlockLOD {
    const message = createBaseSpanBlockLOD();
    message.lodId = object.lodId ?? 0;
    message.tracks = object.tracks?.map((e) => SpanTrack.fromPartial(e)) || [];
    return message;
  },
};

function createBaseBlockSpansReply(): BlockSpansReply {
  return { scopes: {}, lod: undefined, blockId: "", beginMs: 0, endMs: 0 };
}

export const BlockSpansReply = {
  encode(
    message: BlockSpansReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    Object.entries(message.scopes).forEach(([key, value]) => {
      BlockSpansReply_ScopesEntry.encode(
        { key: key as any, value },
        writer.uint32(10).fork()
      ).ldelim();
    });
    if (message.lod !== undefined) {
      SpanBlockLOD.encode(message.lod, writer.uint32(18).fork()).ldelim();
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
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): BlockSpansReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseBlockSpansReply();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          const entry1 = BlockSpansReply_ScopesEntry.decode(
            reader,
            reader.uint32()
          );
          if (entry1.value !== undefined) {
            message.scopes[entry1.key] = entry1.value;
          }
          break;
        case 2:
          message.lod = SpanBlockLOD.decode(reader, reader.uint32());
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
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): BlockSpansReply {
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
      lod: isSet(object.lod) ? SpanBlockLOD.fromJSON(object.lod) : undefined,
      blockId: isSet(object.blockId) ? String(object.blockId) : "",
      beginMs: isSet(object.beginMs) ? Number(object.beginMs) : 0,
      endMs: isSet(object.endMs) ? Number(object.endMs) : 0,
    };
  },

  toJSON(message: BlockSpansReply): unknown {
    const obj: any = {};
    obj.scopes = {};
    if (message.scopes) {
      Object.entries(message.scopes).forEach(([k, v]) => {
        obj.scopes[k] = ScopeDesc.toJSON(v);
      });
    }
    message.lod !== undefined &&
      (obj.lod = message.lod ? SpanBlockLOD.toJSON(message.lod) : undefined);
    message.blockId !== undefined && (obj.blockId = message.blockId);
    message.beginMs !== undefined && (obj.beginMs = message.beginMs);
    message.endMs !== undefined && (obj.endMs = message.endMs);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<BlockSpansReply>, I>>(
    object: I
  ): BlockSpansReply {
    const message = createBaseBlockSpansReply();
    message.scopes = Object.entries(object.scopes ?? {}).reduce<{
      [key: number]: ScopeDesc;
    }>((acc, [key, value]) => {
      if (value !== undefined) {
        acc[Number(key)] = ScopeDesc.fromPartial(value);
      }
      return acc;
    }, {});
    message.lod =
      object.lod !== undefined && object.lod !== null
        ? SpanBlockLOD.fromPartial(object.lod)
        : undefined;
    message.blockId = object.blockId ?? "";
    message.beginMs = object.beginMs ?? 0;
    message.endMs = object.endMs ?? 0;
    return message;
  },
};

function createBaseBlockSpansReply_ScopesEntry(): BlockSpansReply_ScopesEntry {
  return { key: 0, value: undefined };
}

export const BlockSpansReply_ScopesEntry = {
  encode(
    message: BlockSpansReply_ScopesEntry,
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
  ): BlockSpansReply_ScopesEntry {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseBlockSpansReply_ScopesEntry();
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

  fromJSON(object: any): BlockSpansReply_ScopesEntry {
    return {
      key: isSet(object.key) ? Number(object.key) : 0,
      value: isSet(object.value) ? ScopeDesc.fromJSON(object.value) : undefined,
    };
  },

  toJSON(message: BlockSpansReply_ScopesEntry): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = Math.round(message.key));
    message.value !== undefined &&
      (obj.value = message.value ? ScopeDesc.toJSON(message.value) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<BlockSpansReply_ScopesEntry>, I>>(
    object: I
  ): BlockSpansReply_ScopesEntry {
    const message = createBaseBlockSpansReply_ScopesEntry();
    message.key = object.key ?? 0;
    message.value =
      object.value !== undefined && object.value !== null
        ? ScopeDesc.fromPartial(object.value)
        : undefined;
    return message;
  },
};

function createBaseProcessCumulativeCallGraphRequest(): ProcessCumulativeCallGraphRequest {
  return { process: undefined, beginMs: 0, endMs: 0 };
}

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
    const message = createBaseProcessCumulativeCallGraphRequest();
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
    return {
      process: isSet(object.process)
        ? Process.fromJSON(object.process)
        : undefined,
      beginMs: isSet(object.beginMs) ? Number(object.beginMs) : 0,
      endMs: isSet(object.endMs) ? Number(object.endMs) : 0,
    };
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

  fromPartial<
    I extends Exact<DeepPartial<ProcessCumulativeCallGraphRequest>, I>
  >(object: I): ProcessCumulativeCallGraphRequest {
    const message = createBaseProcessCumulativeCallGraphRequest();
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromPartial(object.process)
        : undefined;
    message.beginMs = object.beginMs ?? 0;
    message.endMs = object.endMs ?? 0;
    return message;
  },
};

function createBaseNodeStats(): NodeStats {
  return { sum: 0, min: 0, max: 0, avg: 0, median: 0, count: 0 };
}

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
    const message = createBaseNodeStats();
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
    return {
      sum: isSet(object.sum) ? Number(object.sum) : 0,
      min: isSet(object.min) ? Number(object.min) : 0,
      max: isSet(object.max) ? Number(object.max) : 0,
      avg: isSet(object.avg) ? Number(object.avg) : 0,
      median: isSet(object.median) ? Number(object.median) : 0,
      count: isSet(object.count) ? Number(object.count) : 0,
    };
  },

  toJSON(message: NodeStats): unknown {
    const obj: any = {};
    message.sum !== undefined && (obj.sum = message.sum);
    message.min !== undefined && (obj.min = message.min);
    message.max !== undefined && (obj.max = message.max);
    message.avg !== undefined && (obj.avg = message.avg);
    message.median !== undefined && (obj.median = message.median);
    message.count !== undefined && (obj.count = Math.round(message.count));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<NodeStats>, I>>(
    object: I
  ): NodeStats {
    const message = createBaseNodeStats();
    message.sum = object.sum ?? 0;
    message.min = object.min ?? 0;
    message.max = object.max ?? 0;
    message.avg = object.avg ?? 0;
    message.median = object.median ?? 0;
    message.count = object.count ?? 0;
    return message;
  },
};

function createBaseCallGraphEdge(): CallGraphEdge {
  return { hash: 0, weight: 0 };
}

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
    const message = createBaseCallGraphEdge();
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
    return {
      hash: isSet(object.hash) ? Number(object.hash) : 0,
      weight: isSet(object.weight) ? Number(object.weight) : 0,
    };
  },

  toJSON(message: CallGraphEdge): unknown {
    const obj: any = {};
    message.hash !== undefined && (obj.hash = Math.round(message.hash));
    message.weight !== undefined && (obj.weight = message.weight);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CallGraphEdge>, I>>(
    object: I
  ): CallGraphEdge {
    const message = createBaseCallGraphEdge();
    message.hash = object.hash ?? 0;
    message.weight = object.weight ?? 0;
    return message;
  },
};

function createBaseCumulativeCallGraphNode(): CumulativeCallGraphNode {
  return { hash: 0, stats: undefined, callers: [], callees: [] };
}

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
    const message = createBaseCumulativeCallGraphNode();
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
    return {
      hash: isSet(object.hash) ? Number(object.hash) : 0,
      stats: isSet(object.stats) ? NodeStats.fromJSON(object.stats) : undefined,
      callers: Array.isArray(object?.callers)
        ? object.callers.map((e: any) => CallGraphEdge.fromJSON(e))
        : [],
      callees: Array.isArray(object?.callees)
        ? object.callees.map((e: any) => CallGraphEdge.fromJSON(e))
        : [],
    };
  },

  toJSON(message: CumulativeCallGraphNode): unknown {
    const obj: any = {};
    message.hash !== undefined && (obj.hash = Math.round(message.hash));
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

  fromPartial<I extends Exact<DeepPartial<CumulativeCallGraphNode>, I>>(
    object: I
  ): CumulativeCallGraphNode {
    const message = createBaseCumulativeCallGraphNode();
    message.hash = object.hash ?? 0;
    message.stats =
      object.stats !== undefined && object.stats !== null
        ? NodeStats.fromPartial(object.stats)
        : undefined;
    message.callers =
      object.callers?.map((e) => CallGraphEdge.fromPartial(e)) || [];
    message.callees =
      object.callees?.map((e) => CallGraphEdge.fromPartial(e)) || [];
    return message;
  },
};

function createBaseCumulativeCallGraphReply(): CumulativeCallGraphReply {
  return { scopes: {}, nodes: [] };
}

export const CumulativeCallGraphReply = {
  encode(
    message: CumulativeCallGraphReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    Object.entries(message.scopes).forEach(([key, value]) => {
      CumulativeCallGraphReply_ScopesEntry.encode(
        { key: key as any, value },
        writer.uint32(10).fork()
      ).ldelim();
    });
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
    const message = createBaseCumulativeCallGraphReply();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          const entry1 = CumulativeCallGraphReply_ScopesEntry.decode(
            reader,
            reader.uint32()
          );
          if (entry1.value !== undefined) {
            message.scopes[entry1.key] = entry1.value;
          }
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
      nodes: Array.isArray(object?.nodes)
        ? object.nodes.map((e: any) => CumulativeCallGraphNode.fromJSON(e))
        : [],
    };
  },

  toJSON(message: CumulativeCallGraphReply): unknown {
    const obj: any = {};
    obj.scopes = {};
    if (message.scopes) {
      Object.entries(message.scopes).forEach(([k, v]) => {
        obj.scopes[k] = ScopeDesc.toJSON(v);
      });
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

  fromPartial<I extends Exact<DeepPartial<CumulativeCallGraphReply>, I>>(
    object: I
  ): CumulativeCallGraphReply {
    const message = createBaseCumulativeCallGraphReply();
    message.scopes = Object.entries(object.scopes ?? {}).reduce<{
      [key: number]: ScopeDesc;
    }>((acc, [key, value]) => {
      if (value !== undefined) {
        acc[Number(key)] = ScopeDesc.fromPartial(value);
      }
      return acc;
    }, {});
    message.nodes =
      object.nodes?.map((e) => CumulativeCallGraphNode.fromPartial(e)) || [];
    return message;
  },
};

function createBaseCumulativeCallGraphReply_ScopesEntry(): CumulativeCallGraphReply_ScopesEntry {
  return { key: 0, value: undefined };
}

export const CumulativeCallGraphReply_ScopesEntry = {
  encode(
    message: CumulativeCallGraphReply_ScopesEntry,
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
  ): CumulativeCallGraphReply_ScopesEntry {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseCumulativeCallGraphReply_ScopesEntry();
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

  fromJSON(object: any): CumulativeCallGraphReply_ScopesEntry {
    return {
      key: isSet(object.key) ? Number(object.key) : 0,
      value: isSet(object.value) ? ScopeDesc.fromJSON(object.value) : undefined,
    };
  },

  toJSON(message: CumulativeCallGraphReply_ScopesEntry): unknown {
    const obj: any = {};
    message.key !== undefined && (obj.key = Math.round(message.key));
    message.value !== undefined &&
      (obj.value = message.value ? ScopeDesc.toJSON(message.value) : undefined);
    return obj;
  },

  fromPartial<
    I extends Exact<DeepPartial<CumulativeCallGraphReply_ScopesEntry>, I>
  >(object: I): CumulativeCallGraphReply_ScopesEntry {
    const message = createBaseCumulativeCallGraphReply_ScopesEntry();
    message.key = object.key ?? 0;
    message.value =
      object.value !== undefined && object.value !== null
        ? ScopeDesc.fromPartial(object.value)
        : undefined;
    return message;
  },
};

function createBaseProcessLogRequest(): ProcessLogRequest {
  return { process: undefined, begin: 0, end: 0 };
}

export const ProcessLogRequest = {
  encode(
    message: ProcessLogRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.process !== undefined) {
      Process.encode(message.process, writer.uint32(10).fork()).ldelim();
    }
    if (message.begin !== 0) {
      writer.uint32(16).uint64(message.begin);
    }
    if (message.end !== 0) {
      writer.uint32(24).uint64(message.end);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ProcessLogRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseProcessLogRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.process = Process.decode(reader, reader.uint32());
          break;
        case 2:
          message.begin = longToNumber(reader.uint64() as Long);
          break;
        case 3:
          message.end = longToNumber(reader.uint64() as Long);
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessLogRequest {
    return {
      process: isSet(object.process)
        ? Process.fromJSON(object.process)
        : undefined,
      begin: isSet(object.begin) ? Number(object.begin) : 0,
      end: isSet(object.end) ? Number(object.end) : 0,
    };
  },

  toJSON(message: ProcessLogRequest): unknown {
    const obj: any = {};
    message.process !== undefined &&
      (obj.process = message.process
        ? Process.toJSON(message.process)
        : undefined);
    message.begin !== undefined && (obj.begin = Math.round(message.begin));
    message.end !== undefined && (obj.end = Math.round(message.end));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ProcessLogRequest>, I>>(
    object: I
  ): ProcessLogRequest {
    const message = createBaseProcessLogRequest();
    message.process =
      object.process !== undefined && object.process !== null
        ? Process.fromPartial(object.process)
        : undefined;
    message.begin = object.begin ?? 0;
    message.end = object.end ?? 0;
    return message;
  },
};

function createBaseLogEntry(): LogEntry {
  return { timeMs: 0, msg: "" };
}

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
    const message = createBaseLogEntry();
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
    return {
      timeMs: isSet(object.timeMs) ? Number(object.timeMs) : 0,
      msg: isSet(object.msg) ? String(object.msg) : "",
    };
  },

  toJSON(message: LogEntry): unknown {
    const obj: any = {};
    message.timeMs !== undefined && (obj.timeMs = message.timeMs);
    message.msg !== undefined && (obj.msg = message.msg);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<LogEntry>, I>>(object: I): LogEntry {
    const message = createBaseLogEntry();
    message.timeMs = object.timeMs ?? 0;
    message.msg = object.msg ?? "";
    return message;
  },
};

function createBaseProcessLogReply(): ProcessLogReply {
  return { entries: [], begin: 0, end: 0 };
}

export const ProcessLogReply = {
  encode(
    message: ProcessLogReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.entries) {
      LogEntry.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    if (message.begin !== 0) {
      writer.uint32(16).uint64(message.begin);
    }
    if (message.end !== 0) {
      writer.uint32(24).uint64(message.end);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ProcessLogReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseProcessLogReply();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.entries.push(LogEntry.decode(reader, reader.uint32()));
          break;
        case 2:
          message.begin = longToNumber(reader.uint64() as Long);
          break;
        case 3:
          message.end = longToNumber(reader.uint64() as Long);
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessLogReply {
    return {
      entries: Array.isArray(object?.entries)
        ? object.entries.map((e: any) => LogEntry.fromJSON(e))
        : [],
      begin: isSet(object.begin) ? Number(object.begin) : 0,
      end: isSet(object.end) ? Number(object.end) : 0,
    };
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
    message.begin !== undefined && (obj.begin = Math.round(message.begin));
    message.end !== undefined && (obj.end = Math.round(message.end));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ProcessLogReply>, I>>(
    object: I
  ): ProcessLogReply {
    const message = createBaseProcessLogReply();
    message.entries = object.entries?.map((e) => LogEntry.fromPartial(e)) || [];
    message.begin = object.begin ?? 0;
    message.end = object.end ?? 0;
    return message;
  },
};

function createBaseProcessNbLogEntriesRequest(): ProcessNbLogEntriesRequest {
  return { processId: "" };
}

export const ProcessNbLogEntriesRequest = {
  encode(
    message: ProcessNbLogEntriesRequest,
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
  ): ProcessNbLogEntriesRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseProcessNbLogEntriesRequest();
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

  fromJSON(object: any): ProcessNbLogEntriesRequest {
    return {
      processId: isSet(object.processId) ? String(object.processId) : "",
    };
  },

  toJSON(message: ProcessNbLogEntriesRequest): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ProcessNbLogEntriesRequest>, I>>(
    object: I
  ): ProcessNbLogEntriesRequest {
    const message = createBaseProcessNbLogEntriesRequest();
    message.processId = object.processId ?? "";
    return message;
  },
};

function createBaseProcessNbLogEntriesReply(): ProcessNbLogEntriesReply {
  return { count: 0 };
}

export const ProcessNbLogEntriesReply = {
  encode(
    message: ProcessNbLogEntriesReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.count !== 0) {
      writer.uint32(8).uint64(message.count);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ProcessNbLogEntriesReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseProcessNbLogEntriesReply();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.count = longToNumber(reader.uint64() as Long);
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessNbLogEntriesReply {
    return {
      count: isSet(object.count) ? Number(object.count) : 0,
    };
  },

  toJSON(message: ProcessNbLogEntriesReply): unknown {
    const obj: any = {};
    message.count !== undefined && (obj.count = Math.round(message.count));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ProcessNbLogEntriesReply>, I>>(
    object: I
  ): ProcessNbLogEntriesReply {
    const message = createBaseProcessNbLogEntriesReply();
    message.count = object.count ?? 0;
    return message;
  },
};

function createBaseListProcessChildrenRequest(): ListProcessChildrenRequest {
  return { processId: "" };
}

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
    const message = createBaseListProcessChildrenRequest();
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
    return {
      processId: isSet(object.processId) ? String(object.processId) : "",
    };
  },

  toJSON(message: ListProcessChildrenRequest): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ListProcessChildrenRequest>, I>>(
    object: I
  ): ListProcessChildrenRequest {
    const message = createBaseListProcessChildrenRequest();
    message.processId = object.processId ?? "";
    return message;
  },
};

function createBaseProcessChildrenReply(): ProcessChildrenReply {
  return { processes: [] };
}

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
    const message = createBaseProcessChildrenReply();
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
    return {
      processes: Array.isArray(object?.processes)
        ? object.processes.map((e: any) => Process.fromJSON(e))
        : [],
    };
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

  fromPartial<I extends Exact<DeepPartial<ProcessChildrenReply>, I>>(
    object: I
  ): ProcessChildrenReply {
    const message = createBaseProcessChildrenReply();
    message.processes =
      object.processes?.map((e) => Process.fromPartial(e)) || [];
    return message;
  },
};

function createBaseListProcessMetricsRequest(): ListProcessMetricsRequest {
  return { processId: "" };
}

export const ListProcessMetricsRequest = {
  encode(
    message: ListProcessMetricsRequest,
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
  ): ListProcessMetricsRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseListProcessMetricsRequest();
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

  fromJSON(object: any): ListProcessMetricsRequest {
    return {
      processId: isSet(object.processId) ? String(object.processId) : "",
    };
  },

  toJSON(message: ListProcessMetricsRequest): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ListProcessMetricsRequest>, I>>(
    object: I
  ): ListProcessMetricsRequest {
    const message = createBaseListProcessMetricsRequest();
    message.processId = object.processId ?? "";
    return message;
  },
};

function createBaseMetricDesc(): MetricDesc {
  return { name: "", unit: "" };
}

export const MetricDesc = {
  encode(
    message: MetricDesc,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.unit !== "") {
      writer.uint32(18).string(message.unit);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MetricDesc {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseMetricDesc();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.unit = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): MetricDesc {
    return {
      name: isSet(object.name) ? String(object.name) : "",
      unit: isSet(object.unit) ? String(object.unit) : "",
    };
  },

  toJSON(message: MetricDesc): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.unit !== undefined && (obj.unit = message.unit);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<MetricDesc>, I>>(
    object: I
  ): MetricDesc {
    const message = createBaseMetricDesc();
    message.name = object.name ?? "";
    message.unit = object.unit ?? "";
    return message;
  },
};

function createBaseProcessMetricsReply(): ProcessMetricsReply {
  return { metrics: [], minTimeMs: 0, maxTimeMs: 0 };
}

export const ProcessMetricsReply = {
  encode(
    message: ProcessMetricsReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.metrics) {
      MetricDesc.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    if (message.minTimeMs !== 0) {
      writer.uint32(17).double(message.minTimeMs);
    }
    if (message.maxTimeMs !== 0) {
      writer.uint32(25).double(message.maxTimeMs);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ProcessMetricsReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseProcessMetricsReply();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.metrics.push(MetricDesc.decode(reader, reader.uint32()));
          break;
        case 2:
          message.minTimeMs = reader.double();
          break;
        case 3:
          message.maxTimeMs = reader.double();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessMetricsReply {
    return {
      metrics: Array.isArray(object?.metrics)
        ? object.metrics.map((e: any) => MetricDesc.fromJSON(e))
        : [],
      minTimeMs: isSet(object.minTimeMs) ? Number(object.minTimeMs) : 0,
      maxTimeMs: isSet(object.maxTimeMs) ? Number(object.maxTimeMs) : 0,
    };
  },

  toJSON(message: ProcessMetricsReply): unknown {
    const obj: any = {};
    if (message.metrics) {
      obj.metrics = message.metrics.map((e) =>
        e ? MetricDesc.toJSON(e) : undefined
      );
    } else {
      obj.metrics = [];
    }
    message.minTimeMs !== undefined && (obj.minTimeMs = message.minTimeMs);
    message.maxTimeMs !== undefined && (obj.maxTimeMs = message.maxTimeMs);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ProcessMetricsReply>, I>>(
    object: I
  ): ProcessMetricsReply {
    const message = createBaseProcessMetricsReply();
    message.metrics =
      object.metrics?.map((e) => MetricDesc.fromPartial(e)) || [];
    message.minTimeMs = object.minTimeMs ?? 0;
    message.maxTimeMs = object.maxTimeMs ?? 0;
    return message;
  },
};

function createBaseFetchProcessMetricRequest(): FetchProcessMetricRequest {
  return { processId: "", metricName: "", unit: "", beginMs: 0, endMs: 0 };
}

export const FetchProcessMetricRequest = {
  encode(
    message: FetchProcessMetricRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.processId !== "") {
      writer.uint32(10).string(message.processId);
    }
    if (message.metricName !== "") {
      writer.uint32(18).string(message.metricName);
    }
    if (message.unit !== "") {
      writer.uint32(26).string(message.unit);
    }
    if (message.beginMs !== 0) {
      writer.uint32(33).double(message.beginMs);
    }
    if (message.endMs !== 0) {
      writer.uint32(41).double(message.endMs);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): FetchProcessMetricRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseFetchProcessMetricRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.processId = reader.string();
          break;
        case 2:
          message.metricName = reader.string();
          break;
        case 3:
          message.unit = reader.string();
          break;
        case 4:
          message.beginMs = reader.double();
          break;
        case 5:
          message.endMs = reader.double();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FetchProcessMetricRequest {
    return {
      processId: isSet(object.processId) ? String(object.processId) : "",
      metricName: isSet(object.metricName) ? String(object.metricName) : "",
      unit: isSet(object.unit) ? String(object.unit) : "",
      beginMs: isSet(object.beginMs) ? Number(object.beginMs) : 0,
      endMs: isSet(object.endMs) ? Number(object.endMs) : 0,
    };
  },

  toJSON(message: FetchProcessMetricRequest): unknown {
    const obj: any = {};
    message.processId !== undefined && (obj.processId = message.processId);
    message.metricName !== undefined && (obj.metricName = message.metricName);
    message.unit !== undefined && (obj.unit = message.unit);
    message.beginMs !== undefined && (obj.beginMs = message.beginMs);
    message.endMs !== undefined && (obj.endMs = message.endMs);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FetchProcessMetricRequest>, I>>(
    object: I
  ): FetchProcessMetricRequest {
    const message = createBaseFetchProcessMetricRequest();
    message.processId = object.processId ?? "";
    message.metricName = object.metricName ?? "";
    message.unit = object.unit ?? "";
    message.beginMs = object.beginMs ?? 0;
    message.endMs = object.endMs ?? 0;
    return message;
  },
};

function createBaseMetricDataPoint(): MetricDataPoint {
  return { timeMs: 0, value: 0 };
}

export const MetricDataPoint = {
  encode(
    message: MetricDataPoint,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.timeMs !== 0) {
      writer.uint32(9).double(message.timeMs);
    }
    if (message.value !== 0) {
      writer.uint32(17).double(message.value);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MetricDataPoint {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseMetricDataPoint();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.timeMs = reader.double();
          break;
        case 2:
          message.value = reader.double();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): MetricDataPoint {
    return {
      timeMs: isSet(object.timeMs) ? Number(object.timeMs) : 0,
      value: isSet(object.value) ? Number(object.value) : 0,
    };
  },

  toJSON(message: MetricDataPoint): unknown {
    const obj: any = {};
    message.timeMs !== undefined && (obj.timeMs = message.timeMs);
    message.value !== undefined && (obj.value = message.value);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<MetricDataPoint>, I>>(
    object: I
  ): MetricDataPoint {
    const message = createBaseMetricDataPoint();
    message.timeMs = object.timeMs ?? 0;
    message.value = object.value ?? 0;
    return message;
  },
};

function createBaseProcessMetricReply(): ProcessMetricReply {
  return { points: [] };
}

export const ProcessMetricReply = {
  encode(
    message: ProcessMetricReply,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.points) {
      MetricDataPoint.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ProcessMetricReply {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseProcessMetricReply();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.points.push(MetricDataPoint.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ProcessMetricReply {
    return {
      points: Array.isArray(object?.points)
        ? object.points.map((e: any) => MetricDataPoint.fromJSON(e))
        : [],
    };
  },

  toJSON(message: ProcessMetricReply): unknown {
    const obj: any = {};
    if (message.points) {
      obj.points = message.points.map((e) =>
        e ? MetricDataPoint.toJSON(e) : undefined
      );
    } else {
      obj.points = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ProcessMetricReply>, I>>(
    object: I
  ): ProcessMetricReply {
    const message = createBaseProcessMetricReply();
    message.points =
      object.points?.map((e) => MetricDataPoint.fromPartial(e)) || [];
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
  nb_process_log_entries(
    request: DeepPartial<ProcessNbLogEntriesRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessNbLogEntriesReply>;
  list_process_streams(
    request: DeepPartial<ListProcessStreamsRequest>,
    metadata?: grpc.Metadata
  ): Promise<ListStreamsReply>;
  list_recent_processes(
    request: DeepPartial<RecentProcessesRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessListReply>;
  search_processes(
    request: DeepPartial<SearchProcessRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessListReply>;
  list_stream_blocks(
    request: DeepPartial<ListStreamBlocksRequest>,
    metadata?: grpc.Metadata
  ): Promise<ListStreamBlocksReply>;
  list_process_metrics(
    request: DeepPartial<ListProcessMetricsRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessMetricsReply>;
  fetch_process_metric(
    request: DeepPartial<FetchProcessMetricRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessMetricReply>;
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
    this.nb_process_log_entries = this.nb_process_log_entries.bind(this);
    this.list_process_streams = this.list_process_streams.bind(this);
    this.list_recent_processes = this.list_recent_processes.bind(this);
    this.search_processes = this.search_processes.bind(this);
    this.list_stream_blocks = this.list_stream_blocks.bind(this);
    this.list_process_metrics = this.list_process_metrics.bind(this);
    this.fetch_process_metric = this.fetch_process_metric.bind(this);
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

  nb_process_log_entries(
    request: DeepPartial<ProcessNbLogEntriesRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessNbLogEntriesReply> {
    return this.rpc.unary(
      PerformanceAnalyticsnb_process_log_entriesDesc,
      ProcessNbLogEntriesRequest.fromPartial(request),
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

  search_processes(
    request: DeepPartial<SearchProcessRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessListReply> {
    return this.rpc.unary(
      PerformanceAnalyticssearch_processesDesc,
      SearchProcessRequest.fromPartial(request),
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

  list_process_metrics(
    request: DeepPartial<ListProcessMetricsRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessMetricsReply> {
    return this.rpc.unary(
      PerformanceAnalyticslist_process_metricsDesc,
      ListProcessMetricsRequest.fromPartial(request),
      metadata
    );
  }

  fetch_process_metric(
    request: DeepPartial<FetchProcessMetricRequest>,
    metadata?: grpc.Metadata
  ): Promise<ProcessMetricReply> {
    return this.rpc.unary(
      PerformanceAnalyticsfetch_process_metricDesc,
      FetchProcessMetricRequest.fromPartial(request),
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

export const PerformanceAnalyticsnb_process_log_entriesDesc: UnaryMethodDefinitionish =
  {
    methodName: "nb_process_log_entries",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return ProcessNbLogEntriesRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...ProcessNbLogEntriesReply.decode(data),
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

export const PerformanceAnalyticssearch_processesDesc: UnaryMethodDefinitionish =
  {
    methodName: "search_processes",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return SearchProcessRequest.encode(this).finish();
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

export const PerformanceAnalyticslist_process_metricsDesc: UnaryMethodDefinitionish =
  {
    methodName: "list_process_metrics",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return ListProcessMetricsRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...ProcessMetricsReply.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const PerformanceAnalyticsfetch_process_metricDesc: UnaryMethodDefinitionish =
  {
    methodName: "fetch_process_metric",
    service: PerformanceAnalyticsDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return FetchProcessMetricRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...ProcessMetricReply.decode(data),
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

function isObject(value: any): boolean {
  return typeof value === "object" && value !== null;
}

function isSet(value: any): boolean {
  return value !== null && value !== undefined;
}
