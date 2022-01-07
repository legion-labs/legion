/* eslint-disable */
import Long from "long";
import { grpc } from "@improbable-eng/grpc-web";
import _m0 from "protobufjs/minimal";
import { Empty } from "./google/protobuf/empty";
import { BrowserHeaders } from "browser-headers";
import { Timestamp } from "./google/protobuf/timestamp";

export const protobufPackage = "source_control";

export enum ChangeType {
  EDIT = 0,
  ADD = 1,
  DELETE = 2,
  UNRECOGNIZED = -1,
}

export function changeTypeFromJSON(object: any): ChangeType {
  switch (object) {
    case 0:
    case "EDIT":
      return ChangeType.EDIT;
    case 1:
    case "ADD":
      return ChangeType.ADD;
    case 2:
    case "DELETE":
      return ChangeType.DELETE;
    case -1:
    case "UNRECOGNIZED":
    default:
      return ChangeType.UNRECOGNIZED;
  }
}

export function changeTypeToJSON(object: ChangeType): string {
  switch (object) {
    case ChangeType.EDIT:
      return "EDIT";
    case ChangeType.ADD:
      return "ADD";
    case ChangeType.DELETE:
      return "DELETE";
    default:
      return "UNKNOWN";
  }
}

export interface CreateRepositoryRequest {
  repositoryName: string;
}

export interface CreateRepositoryResponse {
  blobStorageUrl: string;
}

export interface DestroyRepositoryRequest {
  repositoryName: string;
}

export interface DestroyRepositoryResponse {}

export interface GetBlobStorageUrlRequest {
  repositoryName: string;
}

export interface GetBlobStorageUrlResponse {
  blobStorageUrl: string;
}

export interface InsertWorkspaceRequest {
  repositoryName: string;
  workspace: Workspace | undefined;
}

export interface InsertWorkspaceResponse {}

export interface Workspace {
  id: string;
  repositoryAddress: string;
  root: string;
  owner: string;
}

export interface FindBranchRequest {
  repositoryName: string;
  branchName: string;
}

export interface FindBranchResponse {
  value: Branch | undefined;
}

export interface Branch {
  name: string;
  head: string;
  parent: string;
  lockDomainId: string;
}

export interface ReadBranchesRequest {
  repositoryName: string;
}

export interface ReadBranchesResponse {
  value: Branch[];
}

export interface FindBranchesInLockDomainRequest {
  repositoryName: string;
  lockDomainId: string;
}

export interface FindBranchesInLockDomainResponse {
  branch: Branch[];
}

export interface ReadCommitRequest {
  repositoryName: string;
  commitId: string;
}

export interface ReadCommitResponse {
  commit: Commit | undefined;
}

export interface Commit {
  id: string;
  owner: string;
  message: string;
  changes: HashedChange[];
  rootHash: string;
  parents: string[];
  timestamp: Date | undefined;
}

export interface HashedChange {
  relativePath: string;
  hash: string;
  changeType: ChangeType;
}

export interface ReadTreeRequest {
  repositoryName: string;
  treeHash: string;
}

export interface ReadTreeResponse {
  tree: Tree | undefined;
}

export interface TreeNode {
  name: string;
  hash: string;
}

export interface Tree {
  directoryNodes: TreeNode[];
  fileNodes: TreeNode[];
}

export interface InsertLockRequest {
  repositoryName: string;
  lock: Lock | undefined;
}

export interface InsertLockResponse {}

export interface Lock {
  relativePath: string;
  lockDomainId: string;
  workspaceId: string;
  branchName: string;
}

export interface FindLockRequest {
  repositoryName: string;
  lockDomainId: string;
  canonicalRelativePath: string;
}

export interface FindLockResponse {
  value: Lock | undefined;
}

export interface FindLocksInDomainRequest {
  repositoryName: string;
  lockDomainId: string;
}

export interface FindLocksInDomainResponse {
  locks: Lock[];
}

export interface SaveTreeRequest {
  repositoryName: string;
  tree: Tree | undefined;
  hash: string;
}

export interface SaveTreeResponse {}

export interface InsertCommitRequest {
  repositoryName: string;
  commit: Commit | undefined;
}

export interface InsertCommitResponse {}

export interface CommitToBranchRequest {
  repositoryName: string;
  commit: Commit | undefined;
  branch: Branch | undefined;
}

export interface CommitToBranchResponse {}

export interface CommitExistsRequest {
  repositoryName: string;
  commitId: string;
}

export interface CommitExistsResponse {
  exists: boolean;
}

export interface UpdateBranchRequest {
  repositoryName: string;
  branch: Branch | undefined;
}

export interface UpdateBranchResponse {}

export interface InsertBranchRequest {
  repositoryName: string;
  branch: Branch | undefined;
}

export interface InsertBranchResponse {}

export interface ClearLockRequest {
  repositoryName: string;
  lockDomainId: string;
  canonicalRelativePath: string;
}

export interface ClearLockResponse {}

export interface CountLocksInDomainRequest {
  repositoryName: string;
  lockDomainId: string;
}

export interface CountLocksInDomainResponse {
  count: number;
}

const baseCreateRepositoryRequest: object = { repositoryName: "" };

export const CreateRepositoryRequest = {
  encode(
    message: CreateRepositoryRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CreateRepositoryRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseCreateRepositoryRequest,
    } as CreateRepositoryRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CreateRepositoryRequest {
    const message = {
      ...baseCreateRepositoryRequest,
    } as CreateRepositoryRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    return message;
  },

  toJSON(message: CreateRepositoryRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CreateRepositoryRequest>, I>>(
    object: I
  ): CreateRepositoryRequest {
    const message = {
      ...baseCreateRepositoryRequest,
    } as CreateRepositoryRequest;
    message.repositoryName = object.repositoryName ?? "";
    return message;
  },
};

const baseCreateRepositoryResponse: object = { blobStorageUrl: "" };

export const CreateRepositoryResponse = {
  encode(
    message: CreateRepositoryResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.blobStorageUrl !== "") {
      writer.uint32(10).string(message.blobStorageUrl);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CreateRepositoryResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseCreateRepositoryResponse,
    } as CreateRepositoryResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.blobStorageUrl = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CreateRepositoryResponse {
    const message = {
      ...baseCreateRepositoryResponse,
    } as CreateRepositoryResponse;
    message.blobStorageUrl =
      object.blobStorageUrl !== undefined && object.blobStorageUrl !== null
        ? String(object.blobStorageUrl)
        : "";
    return message;
  },

  toJSON(message: CreateRepositoryResponse): unknown {
    const obj: any = {};
    message.blobStorageUrl !== undefined &&
      (obj.blobStorageUrl = message.blobStorageUrl);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CreateRepositoryResponse>, I>>(
    object: I
  ): CreateRepositoryResponse {
    const message = {
      ...baseCreateRepositoryResponse,
    } as CreateRepositoryResponse;
    message.blobStorageUrl = object.blobStorageUrl ?? "";
    return message;
  },
};

const baseDestroyRepositoryRequest: object = { repositoryName: "" };

export const DestroyRepositoryRequest = {
  encode(
    message: DestroyRepositoryRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): DestroyRepositoryRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseDestroyRepositoryRequest,
    } as DestroyRepositoryRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): DestroyRepositoryRequest {
    const message = {
      ...baseDestroyRepositoryRequest,
    } as DestroyRepositoryRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    return message;
  },

  toJSON(message: DestroyRepositoryRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<DestroyRepositoryRequest>, I>>(
    object: I
  ): DestroyRepositoryRequest {
    const message = {
      ...baseDestroyRepositoryRequest,
    } as DestroyRepositoryRequest;
    message.repositoryName = object.repositoryName ?? "";
    return message;
  },
};

const baseDestroyRepositoryResponse: object = {};

export const DestroyRepositoryResponse = {
  encode(
    _: DestroyRepositoryResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): DestroyRepositoryResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseDestroyRepositoryResponse,
    } as DestroyRepositoryResponse;
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

  fromJSON(_: any): DestroyRepositoryResponse {
    const message = {
      ...baseDestroyRepositoryResponse,
    } as DestroyRepositoryResponse;
    return message;
  },

  toJSON(_: DestroyRepositoryResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<DestroyRepositoryResponse>, I>>(
    _: I
  ): DestroyRepositoryResponse {
    const message = {
      ...baseDestroyRepositoryResponse,
    } as DestroyRepositoryResponse;
    return message;
  },
};

const baseGetBlobStorageUrlRequest: object = { repositoryName: "" };

export const GetBlobStorageUrlRequest = {
  encode(
    message: GetBlobStorageUrlRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): GetBlobStorageUrlRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseGetBlobStorageUrlRequest,
    } as GetBlobStorageUrlRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): GetBlobStorageUrlRequest {
    const message = {
      ...baseGetBlobStorageUrlRequest,
    } as GetBlobStorageUrlRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    return message;
  },

  toJSON(message: GetBlobStorageUrlRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<GetBlobStorageUrlRequest>, I>>(
    object: I
  ): GetBlobStorageUrlRequest {
    const message = {
      ...baseGetBlobStorageUrlRequest,
    } as GetBlobStorageUrlRequest;
    message.repositoryName = object.repositoryName ?? "";
    return message;
  },
};

const baseGetBlobStorageUrlResponse: object = { blobStorageUrl: "" };

export const GetBlobStorageUrlResponse = {
  encode(
    message: GetBlobStorageUrlResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.blobStorageUrl !== "") {
      writer.uint32(10).string(message.blobStorageUrl);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): GetBlobStorageUrlResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseGetBlobStorageUrlResponse,
    } as GetBlobStorageUrlResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.blobStorageUrl = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): GetBlobStorageUrlResponse {
    const message = {
      ...baseGetBlobStorageUrlResponse,
    } as GetBlobStorageUrlResponse;
    message.blobStorageUrl =
      object.blobStorageUrl !== undefined && object.blobStorageUrl !== null
        ? String(object.blobStorageUrl)
        : "";
    return message;
  },

  toJSON(message: GetBlobStorageUrlResponse): unknown {
    const obj: any = {};
    message.blobStorageUrl !== undefined &&
      (obj.blobStorageUrl = message.blobStorageUrl);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<GetBlobStorageUrlResponse>, I>>(
    object: I
  ): GetBlobStorageUrlResponse {
    const message = {
      ...baseGetBlobStorageUrlResponse,
    } as GetBlobStorageUrlResponse;
    message.blobStorageUrl = object.blobStorageUrl ?? "";
    return message;
  },
};

const baseInsertWorkspaceRequest: object = { repositoryName: "" };

export const InsertWorkspaceRequest = {
  encode(
    message: InsertWorkspaceRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.workspace !== undefined) {
      Workspace.encode(message.workspace, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): InsertWorkspaceRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInsertWorkspaceRequest } as InsertWorkspaceRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.workspace = Workspace.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InsertWorkspaceRequest {
    const message = { ...baseInsertWorkspaceRequest } as InsertWorkspaceRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.workspace =
      object.workspace !== undefined && object.workspace !== null
        ? Workspace.fromJSON(object.workspace)
        : undefined;
    return message;
  },

  toJSON(message: InsertWorkspaceRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.workspace !== undefined &&
      (obj.workspace = message.workspace
        ? Workspace.toJSON(message.workspace)
        : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertWorkspaceRequest>, I>>(
    object: I
  ): InsertWorkspaceRequest {
    const message = { ...baseInsertWorkspaceRequest } as InsertWorkspaceRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.workspace =
      object.workspace !== undefined && object.workspace !== null
        ? Workspace.fromPartial(object.workspace)
        : undefined;
    return message;
  },
};

const baseInsertWorkspaceResponse: object = {};

export const InsertWorkspaceResponse = {
  encode(
    _: InsertWorkspaceResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): InsertWorkspaceResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseInsertWorkspaceResponse,
    } as InsertWorkspaceResponse;
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

  fromJSON(_: any): InsertWorkspaceResponse {
    const message = {
      ...baseInsertWorkspaceResponse,
    } as InsertWorkspaceResponse;
    return message;
  },

  toJSON(_: InsertWorkspaceResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertWorkspaceResponse>, I>>(
    _: I
  ): InsertWorkspaceResponse {
    const message = {
      ...baseInsertWorkspaceResponse,
    } as InsertWorkspaceResponse;
    return message;
  },
};

const baseWorkspace: object = {
  id: "",
  repositoryAddress: "",
  root: "",
  owner: "",
};

export const Workspace = {
  encode(
    message: Workspace,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.repositoryAddress !== "") {
      writer.uint32(18).string(message.repositoryAddress);
    }
    if (message.root !== "") {
      writer.uint32(26).string(message.root);
    }
    if (message.owner !== "") {
      writer.uint32(34).string(message.owner);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Workspace {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseWorkspace } as Workspace;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.repositoryAddress = reader.string();
          break;
        case 3:
          message.root = reader.string();
          break;
        case 4:
          message.owner = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Workspace {
    const message = { ...baseWorkspace } as Workspace;
    message.id =
      object.id !== undefined && object.id !== null ? String(object.id) : "";
    message.repositoryAddress =
      object.repositoryAddress !== undefined &&
      object.repositoryAddress !== null
        ? String(object.repositoryAddress)
        : "";
    message.root =
      object.root !== undefined && object.root !== null
        ? String(object.root)
        : "";
    message.owner =
      object.owner !== undefined && object.owner !== null
        ? String(object.owner)
        : "";
    return message;
  },

  toJSON(message: Workspace): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.repositoryAddress !== undefined &&
      (obj.repositoryAddress = message.repositoryAddress);
    message.root !== undefined && (obj.root = message.root);
    message.owner !== undefined && (obj.owner = message.owner);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Workspace>, I>>(
    object: I
  ): Workspace {
    const message = { ...baseWorkspace } as Workspace;
    message.id = object.id ?? "";
    message.repositoryAddress = object.repositoryAddress ?? "";
    message.root = object.root ?? "";
    message.owner = object.owner ?? "";
    return message;
  },
};

const baseFindBranchRequest: object = { repositoryName: "", branchName: "" };

export const FindBranchRequest = {
  encode(
    message: FindBranchRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.branchName !== "") {
      writer.uint32(18).string(message.branchName);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FindBranchRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseFindBranchRequest } as FindBranchRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.branchName = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindBranchRequest {
    const message = { ...baseFindBranchRequest } as FindBranchRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.branchName =
      object.branchName !== undefined && object.branchName !== null
        ? String(object.branchName)
        : "";
    return message;
  },

  toJSON(message: FindBranchRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.branchName !== undefined && (obj.branchName = message.branchName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FindBranchRequest>, I>>(
    object: I
  ): FindBranchRequest {
    const message = { ...baseFindBranchRequest } as FindBranchRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.branchName = object.branchName ?? "";
    return message;
  },
};

const baseFindBranchResponse: object = {};

export const FindBranchResponse = {
  encode(
    message: FindBranchResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.value !== undefined) {
      Branch.encode(message.value, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FindBranchResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseFindBranchResponse } as FindBranchResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.value = Branch.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindBranchResponse {
    const message = { ...baseFindBranchResponse } as FindBranchResponse;
    message.value =
      object.value !== undefined && object.value !== null
        ? Branch.fromJSON(object.value)
        : undefined;
    return message;
  },

  toJSON(message: FindBranchResponse): unknown {
    const obj: any = {};
    message.value !== undefined &&
      (obj.value = message.value ? Branch.toJSON(message.value) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FindBranchResponse>, I>>(
    object: I
  ): FindBranchResponse {
    const message = { ...baseFindBranchResponse } as FindBranchResponse;
    message.value =
      object.value !== undefined && object.value !== null
        ? Branch.fromPartial(object.value)
        : undefined;
    return message;
  },
};

const baseBranch: object = { name: "", head: "", parent: "", lockDomainId: "" };

export const Branch = {
  encode(
    message: Branch,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.head !== "") {
      writer.uint32(18).string(message.head);
    }
    if (message.parent !== "") {
      writer.uint32(26).string(message.parent);
    }
    if (message.lockDomainId !== "") {
      writer.uint32(34).string(message.lockDomainId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Branch {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseBranch } as Branch;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.head = reader.string();
          break;
        case 3:
          message.parent = reader.string();
          break;
        case 4:
          message.lockDomainId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Branch {
    const message = { ...baseBranch } as Branch;
    message.name =
      object.name !== undefined && object.name !== null
        ? String(object.name)
        : "";
    message.head =
      object.head !== undefined && object.head !== null
        ? String(object.head)
        : "";
    message.parent =
      object.parent !== undefined && object.parent !== null
        ? String(object.parent)
        : "";
    message.lockDomainId =
      object.lockDomainId !== undefined && object.lockDomainId !== null
        ? String(object.lockDomainId)
        : "";
    return message;
  },

  toJSON(message: Branch): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.head !== undefined && (obj.head = message.head);
    message.parent !== undefined && (obj.parent = message.parent);
    message.lockDomainId !== undefined &&
      (obj.lockDomainId = message.lockDomainId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Branch>, I>>(object: I): Branch {
    const message = { ...baseBranch } as Branch;
    message.name = object.name ?? "";
    message.head = object.head ?? "";
    message.parent = object.parent ?? "";
    message.lockDomainId = object.lockDomainId ?? "";
    return message;
  },
};

const baseReadBranchesRequest: object = { repositoryName: "" };

export const ReadBranchesRequest = {
  encode(
    message: ReadBranchesRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ReadBranchesRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseReadBranchesRequest } as ReadBranchesRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReadBranchesRequest {
    const message = { ...baseReadBranchesRequest } as ReadBranchesRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    return message;
  },

  toJSON(message: ReadBranchesRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ReadBranchesRequest>, I>>(
    object: I
  ): ReadBranchesRequest {
    const message = { ...baseReadBranchesRequest } as ReadBranchesRequest;
    message.repositoryName = object.repositoryName ?? "";
    return message;
  },
};

const baseReadBranchesResponse: object = {};

export const ReadBranchesResponse = {
  encode(
    message: ReadBranchesResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.value) {
      Branch.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): ReadBranchesResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseReadBranchesResponse } as ReadBranchesResponse;
    message.value = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.value.push(Branch.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReadBranchesResponse {
    const message = { ...baseReadBranchesResponse } as ReadBranchesResponse;
    message.value = (object.value ?? []).map((e: any) => Branch.fromJSON(e));
    return message;
  },

  toJSON(message: ReadBranchesResponse): unknown {
    const obj: any = {};
    if (message.value) {
      obj.value = message.value.map((e) => (e ? Branch.toJSON(e) : undefined));
    } else {
      obj.value = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ReadBranchesResponse>, I>>(
    object: I
  ): ReadBranchesResponse {
    const message = { ...baseReadBranchesResponse } as ReadBranchesResponse;
    message.value = object.value?.map((e) => Branch.fromPartial(e)) || [];
    return message;
  },
};

const baseFindBranchesInLockDomainRequest: object = {
  repositoryName: "",
  lockDomainId: "",
};

export const FindBranchesInLockDomainRequest = {
  encode(
    message: FindBranchesInLockDomainRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.lockDomainId !== "") {
      writer.uint32(18).string(message.lockDomainId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): FindBranchesInLockDomainRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseFindBranchesInLockDomainRequest,
    } as FindBranchesInLockDomainRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.lockDomainId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindBranchesInLockDomainRequest {
    const message = {
      ...baseFindBranchesInLockDomainRequest,
    } as FindBranchesInLockDomainRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.lockDomainId =
      object.lockDomainId !== undefined && object.lockDomainId !== null
        ? String(object.lockDomainId)
        : "";
    return message;
  },

  toJSON(message: FindBranchesInLockDomainRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.lockDomainId !== undefined &&
      (obj.lockDomainId = message.lockDomainId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FindBranchesInLockDomainRequest>, I>>(
    object: I
  ): FindBranchesInLockDomainRequest {
    const message = {
      ...baseFindBranchesInLockDomainRequest,
    } as FindBranchesInLockDomainRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.lockDomainId = object.lockDomainId ?? "";
    return message;
  },
};

const baseFindBranchesInLockDomainResponse: object = {};

export const FindBranchesInLockDomainResponse = {
  encode(
    message: FindBranchesInLockDomainResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.branch) {
      Branch.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): FindBranchesInLockDomainResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseFindBranchesInLockDomainResponse,
    } as FindBranchesInLockDomainResponse;
    message.branch = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.branch.push(Branch.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindBranchesInLockDomainResponse {
    const message = {
      ...baseFindBranchesInLockDomainResponse,
    } as FindBranchesInLockDomainResponse;
    message.branch = (object.branch ?? []).map((e: any) => Branch.fromJSON(e));
    return message;
  },

  toJSON(message: FindBranchesInLockDomainResponse): unknown {
    const obj: any = {};
    if (message.branch) {
      obj.branch = message.branch.map((e) =>
        e ? Branch.toJSON(e) : undefined
      );
    } else {
      obj.branch = [];
    }
    return obj;
  },

  fromPartial<
    I extends Exact<DeepPartial<FindBranchesInLockDomainResponse>, I>
  >(object: I): FindBranchesInLockDomainResponse {
    const message = {
      ...baseFindBranchesInLockDomainResponse,
    } as FindBranchesInLockDomainResponse;
    message.branch = object.branch?.map((e) => Branch.fromPartial(e)) || [];
    return message;
  },
};

const baseReadCommitRequest: object = { repositoryName: "", commitId: "" };

export const ReadCommitRequest = {
  encode(
    message: ReadCommitRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.commitId !== "") {
      writer.uint32(18).string(message.commitId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ReadCommitRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseReadCommitRequest } as ReadCommitRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.commitId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReadCommitRequest {
    const message = { ...baseReadCommitRequest } as ReadCommitRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.commitId =
      object.commitId !== undefined && object.commitId !== null
        ? String(object.commitId)
        : "";
    return message;
  },

  toJSON(message: ReadCommitRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.commitId !== undefined && (obj.commitId = message.commitId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ReadCommitRequest>, I>>(
    object: I
  ): ReadCommitRequest {
    const message = { ...baseReadCommitRequest } as ReadCommitRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.commitId = object.commitId ?? "";
    return message;
  },
};

const baseReadCommitResponse: object = {};

export const ReadCommitResponse = {
  encode(
    message: ReadCommitResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.commit !== undefined) {
      Commit.encode(message.commit, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ReadCommitResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseReadCommitResponse } as ReadCommitResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.commit = Commit.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReadCommitResponse {
    const message = { ...baseReadCommitResponse } as ReadCommitResponse;
    message.commit =
      object.commit !== undefined && object.commit !== null
        ? Commit.fromJSON(object.commit)
        : undefined;
    return message;
  },

  toJSON(message: ReadCommitResponse): unknown {
    const obj: any = {};
    message.commit !== undefined &&
      (obj.commit = message.commit ? Commit.toJSON(message.commit) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ReadCommitResponse>, I>>(
    object: I
  ): ReadCommitResponse {
    const message = { ...baseReadCommitResponse } as ReadCommitResponse;
    message.commit =
      object.commit !== undefined && object.commit !== null
        ? Commit.fromPartial(object.commit)
        : undefined;
    return message;
  },
};

const baseCommit: object = {
  id: "",
  owner: "",
  message: "",
  rootHash: "",
  parents: "",
};

export const Commit = {
  encode(
    message: Commit,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.owner !== "") {
      writer.uint32(18).string(message.owner);
    }
    if (message.message !== "") {
      writer.uint32(26).string(message.message);
    }
    for (const v of message.changes) {
      HashedChange.encode(v!, writer.uint32(34).fork()).ldelim();
    }
    if (message.rootHash !== "") {
      writer.uint32(42).string(message.rootHash);
    }
    for (const v of message.parents) {
      writer.uint32(50).string(v!);
    }
    if (message.timestamp !== undefined) {
      Timestamp.encode(
        toTimestamp(message.timestamp),
        writer.uint32(58).fork()
      ).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Commit {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCommit } as Commit;
    message.changes = [];
    message.parents = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.owner = reader.string();
          break;
        case 3:
          message.message = reader.string();
          break;
        case 4:
          message.changes.push(HashedChange.decode(reader, reader.uint32()));
          break;
        case 5:
          message.rootHash = reader.string();
          break;
        case 6:
          message.parents.push(reader.string());
          break;
        case 7:
          message.timestamp = fromTimestamp(
            Timestamp.decode(reader, reader.uint32())
          );
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Commit {
    const message = { ...baseCommit } as Commit;
    message.id =
      object.id !== undefined && object.id !== null ? String(object.id) : "";
    message.owner =
      object.owner !== undefined && object.owner !== null
        ? String(object.owner)
        : "";
    message.message =
      object.message !== undefined && object.message !== null
        ? String(object.message)
        : "";
    message.changes = (object.changes ?? []).map((e: any) =>
      HashedChange.fromJSON(e)
    );
    message.rootHash =
      object.rootHash !== undefined && object.rootHash !== null
        ? String(object.rootHash)
        : "";
    message.parents = (object.parents ?? []).map((e: any) => String(e));
    message.timestamp =
      object.timestamp !== undefined && object.timestamp !== null
        ? fromJsonTimestamp(object.timestamp)
        : undefined;
    return message;
  },

  toJSON(message: Commit): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.owner !== undefined && (obj.owner = message.owner);
    message.message !== undefined && (obj.message = message.message);
    if (message.changes) {
      obj.changes = message.changes.map((e) =>
        e ? HashedChange.toJSON(e) : undefined
      );
    } else {
      obj.changes = [];
    }
    message.rootHash !== undefined && (obj.rootHash = message.rootHash);
    if (message.parents) {
      obj.parents = message.parents.map((e) => e);
    } else {
      obj.parents = [];
    }
    message.timestamp !== undefined &&
      (obj.timestamp = message.timestamp.toISOString());
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Commit>, I>>(object: I): Commit {
    const message = { ...baseCommit } as Commit;
    message.id = object.id ?? "";
    message.owner = object.owner ?? "";
    message.message = object.message ?? "";
    message.changes =
      object.changes?.map((e) => HashedChange.fromPartial(e)) || [];
    message.rootHash = object.rootHash ?? "";
    message.parents = object.parents?.map((e) => e) || [];
    message.timestamp = object.timestamp ?? undefined;
    return message;
  },
};

const baseHashedChange: object = { relativePath: "", hash: "", changeType: 0 };

export const HashedChange = {
  encode(
    message: HashedChange,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.relativePath !== "") {
      writer.uint32(10).string(message.relativePath);
    }
    if (message.hash !== "") {
      writer.uint32(18).string(message.hash);
    }
    if (message.changeType !== 0) {
      writer.uint32(24).int32(message.changeType);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): HashedChange {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseHashedChange } as HashedChange;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.relativePath = reader.string();
          break;
        case 2:
          message.hash = reader.string();
          break;
        case 3:
          message.changeType = reader.int32() as any;
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): HashedChange {
    const message = { ...baseHashedChange } as HashedChange;
    message.relativePath =
      object.relativePath !== undefined && object.relativePath !== null
        ? String(object.relativePath)
        : "";
    message.hash =
      object.hash !== undefined && object.hash !== null
        ? String(object.hash)
        : "";
    message.changeType =
      object.changeType !== undefined && object.changeType !== null
        ? changeTypeFromJSON(object.changeType)
        : 0;
    return message;
  },

  toJSON(message: HashedChange): unknown {
    const obj: any = {};
    message.relativePath !== undefined &&
      (obj.relativePath = message.relativePath);
    message.hash !== undefined && (obj.hash = message.hash);
    message.changeType !== undefined &&
      (obj.changeType = changeTypeToJSON(message.changeType));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<HashedChange>, I>>(
    object: I
  ): HashedChange {
    const message = { ...baseHashedChange } as HashedChange;
    message.relativePath = object.relativePath ?? "";
    message.hash = object.hash ?? "";
    message.changeType = object.changeType ?? 0;
    return message;
  },
};

const baseReadTreeRequest: object = { repositoryName: "", treeHash: "" };

export const ReadTreeRequest = {
  encode(
    message: ReadTreeRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.treeHash !== "") {
      writer.uint32(18).string(message.treeHash);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ReadTreeRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseReadTreeRequest } as ReadTreeRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.treeHash = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReadTreeRequest {
    const message = { ...baseReadTreeRequest } as ReadTreeRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.treeHash =
      object.treeHash !== undefined && object.treeHash !== null
        ? String(object.treeHash)
        : "";
    return message;
  },

  toJSON(message: ReadTreeRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.treeHash !== undefined && (obj.treeHash = message.treeHash);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ReadTreeRequest>, I>>(
    object: I
  ): ReadTreeRequest {
    const message = { ...baseReadTreeRequest } as ReadTreeRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.treeHash = object.treeHash ?? "";
    return message;
  },
};

const baseReadTreeResponse: object = {};

export const ReadTreeResponse = {
  encode(
    message: ReadTreeResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.tree !== undefined) {
      Tree.encode(message.tree, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ReadTreeResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseReadTreeResponse } as ReadTreeResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.tree = Tree.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ReadTreeResponse {
    const message = { ...baseReadTreeResponse } as ReadTreeResponse;
    message.tree =
      object.tree !== undefined && object.tree !== null
        ? Tree.fromJSON(object.tree)
        : undefined;
    return message;
  },

  toJSON(message: ReadTreeResponse): unknown {
    const obj: any = {};
    message.tree !== undefined &&
      (obj.tree = message.tree ? Tree.toJSON(message.tree) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ReadTreeResponse>, I>>(
    object: I
  ): ReadTreeResponse {
    const message = { ...baseReadTreeResponse } as ReadTreeResponse;
    message.tree =
      object.tree !== undefined && object.tree !== null
        ? Tree.fromPartial(object.tree)
        : undefined;
    return message;
  },
};

const baseTreeNode: object = { name: "", hash: "" };

export const TreeNode = {
  encode(
    message: TreeNode,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.hash !== "") {
      writer.uint32(18).string(message.hash);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TreeNode {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseTreeNode } as TreeNode;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.name = reader.string();
          break;
        case 2:
          message.hash = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): TreeNode {
    const message = { ...baseTreeNode } as TreeNode;
    message.name =
      object.name !== undefined && object.name !== null
        ? String(object.name)
        : "";
    message.hash =
      object.hash !== undefined && object.hash !== null
        ? String(object.hash)
        : "";
    return message;
  },

  toJSON(message: TreeNode): unknown {
    const obj: any = {};
    message.name !== undefined && (obj.name = message.name);
    message.hash !== undefined && (obj.hash = message.hash);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<TreeNode>, I>>(object: I): TreeNode {
    const message = { ...baseTreeNode } as TreeNode;
    message.name = object.name ?? "";
    message.hash = object.hash ?? "";
    return message;
  },
};

const baseTree: object = {};

export const Tree = {
  encode(message: Tree, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.directoryNodes) {
      TreeNode.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.fileNodes) {
      TreeNode.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Tree {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseTree } as Tree;
    message.directoryNodes = [];
    message.fileNodes = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.directoryNodes.push(TreeNode.decode(reader, reader.uint32()));
          break;
        case 2:
          message.fileNodes.push(TreeNode.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Tree {
    const message = { ...baseTree } as Tree;
    message.directoryNodes = (object.directoryNodes ?? []).map((e: any) =>
      TreeNode.fromJSON(e)
    );
    message.fileNodes = (object.fileNodes ?? []).map((e: any) =>
      TreeNode.fromJSON(e)
    );
    return message;
  },

  toJSON(message: Tree): unknown {
    const obj: any = {};
    if (message.directoryNodes) {
      obj.directoryNodes = message.directoryNodes.map((e) =>
        e ? TreeNode.toJSON(e) : undefined
      );
    } else {
      obj.directoryNodes = [];
    }
    if (message.fileNodes) {
      obj.fileNodes = message.fileNodes.map((e) =>
        e ? TreeNode.toJSON(e) : undefined
      );
    } else {
      obj.fileNodes = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Tree>, I>>(object: I): Tree {
    const message = { ...baseTree } as Tree;
    message.directoryNodes =
      object.directoryNodes?.map((e) => TreeNode.fromPartial(e)) || [];
    message.fileNodes =
      object.fileNodes?.map((e) => TreeNode.fromPartial(e)) || [];
    return message;
  },
};

const baseInsertLockRequest: object = { repositoryName: "" };

export const InsertLockRequest = {
  encode(
    message: InsertLockRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.lock !== undefined) {
      Lock.encode(message.lock, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): InsertLockRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInsertLockRequest } as InsertLockRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.lock = Lock.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InsertLockRequest {
    const message = { ...baseInsertLockRequest } as InsertLockRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.lock =
      object.lock !== undefined && object.lock !== null
        ? Lock.fromJSON(object.lock)
        : undefined;
    return message;
  },

  toJSON(message: InsertLockRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.lock !== undefined &&
      (obj.lock = message.lock ? Lock.toJSON(message.lock) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertLockRequest>, I>>(
    object: I
  ): InsertLockRequest {
    const message = { ...baseInsertLockRequest } as InsertLockRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.lock =
      object.lock !== undefined && object.lock !== null
        ? Lock.fromPartial(object.lock)
        : undefined;
    return message;
  },
};

const baseInsertLockResponse: object = {};

export const InsertLockResponse = {
  encode(
    _: InsertLockResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): InsertLockResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInsertLockResponse } as InsertLockResponse;
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

  fromJSON(_: any): InsertLockResponse {
    const message = { ...baseInsertLockResponse } as InsertLockResponse;
    return message;
  },

  toJSON(_: InsertLockResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertLockResponse>, I>>(
    _: I
  ): InsertLockResponse {
    const message = { ...baseInsertLockResponse } as InsertLockResponse;
    return message;
  },
};

const baseLock: object = {
  relativePath: "",
  lockDomainId: "",
  workspaceId: "",
  branchName: "",
};

export const Lock = {
  encode(message: Lock, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.relativePath !== "") {
      writer.uint32(10).string(message.relativePath);
    }
    if (message.lockDomainId !== "") {
      writer.uint32(18).string(message.lockDomainId);
    }
    if (message.workspaceId !== "") {
      writer.uint32(26).string(message.workspaceId);
    }
    if (message.branchName !== "") {
      writer.uint32(34).string(message.branchName);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Lock {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseLock } as Lock;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.relativePath = reader.string();
          break;
        case 2:
          message.lockDomainId = reader.string();
          break;
        case 3:
          message.workspaceId = reader.string();
          break;
        case 4:
          message.branchName = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Lock {
    const message = { ...baseLock } as Lock;
    message.relativePath =
      object.relativePath !== undefined && object.relativePath !== null
        ? String(object.relativePath)
        : "";
    message.lockDomainId =
      object.lockDomainId !== undefined && object.lockDomainId !== null
        ? String(object.lockDomainId)
        : "";
    message.workspaceId =
      object.workspaceId !== undefined && object.workspaceId !== null
        ? String(object.workspaceId)
        : "";
    message.branchName =
      object.branchName !== undefined && object.branchName !== null
        ? String(object.branchName)
        : "";
    return message;
  },

  toJSON(message: Lock): unknown {
    const obj: any = {};
    message.relativePath !== undefined &&
      (obj.relativePath = message.relativePath);
    message.lockDomainId !== undefined &&
      (obj.lockDomainId = message.lockDomainId);
    message.workspaceId !== undefined &&
      (obj.workspaceId = message.workspaceId);
    message.branchName !== undefined && (obj.branchName = message.branchName);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<Lock>, I>>(object: I): Lock {
    const message = { ...baseLock } as Lock;
    message.relativePath = object.relativePath ?? "";
    message.lockDomainId = object.lockDomainId ?? "";
    message.workspaceId = object.workspaceId ?? "";
    message.branchName = object.branchName ?? "";
    return message;
  },
};

const baseFindLockRequest: object = {
  repositoryName: "",
  lockDomainId: "",
  canonicalRelativePath: "",
};

export const FindLockRequest = {
  encode(
    message: FindLockRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.lockDomainId !== "") {
      writer.uint32(18).string(message.lockDomainId);
    }
    if (message.canonicalRelativePath !== "") {
      writer.uint32(26).string(message.canonicalRelativePath);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FindLockRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseFindLockRequest } as FindLockRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.lockDomainId = reader.string();
          break;
        case 3:
          message.canonicalRelativePath = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindLockRequest {
    const message = { ...baseFindLockRequest } as FindLockRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.lockDomainId =
      object.lockDomainId !== undefined && object.lockDomainId !== null
        ? String(object.lockDomainId)
        : "";
    message.canonicalRelativePath =
      object.canonicalRelativePath !== undefined &&
      object.canonicalRelativePath !== null
        ? String(object.canonicalRelativePath)
        : "";
    return message;
  },

  toJSON(message: FindLockRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.lockDomainId !== undefined &&
      (obj.lockDomainId = message.lockDomainId);
    message.canonicalRelativePath !== undefined &&
      (obj.canonicalRelativePath = message.canonicalRelativePath);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FindLockRequest>, I>>(
    object: I
  ): FindLockRequest {
    const message = { ...baseFindLockRequest } as FindLockRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.lockDomainId = object.lockDomainId ?? "";
    message.canonicalRelativePath = object.canonicalRelativePath ?? "";
    return message;
  },
};

const baseFindLockResponse: object = {};

export const FindLockResponse = {
  encode(
    message: FindLockResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.value !== undefined) {
      Lock.encode(message.value, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): FindLockResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseFindLockResponse } as FindLockResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.value = Lock.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindLockResponse {
    const message = { ...baseFindLockResponse } as FindLockResponse;
    message.value =
      object.value !== undefined && object.value !== null
        ? Lock.fromJSON(object.value)
        : undefined;
    return message;
  },

  toJSON(message: FindLockResponse): unknown {
    const obj: any = {};
    message.value !== undefined &&
      (obj.value = message.value ? Lock.toJSON(message.value) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FindLockResponse>, I>>(
    object: I
  ): FindLockResponse {
    const message = { ...baseFindLockResponse } as FindLockResponse;
    message.value =
      object.value !== undefined && object.value !== null
        ? Lock.fromPartial(object.value)
        : undefined;
    return message;
  },
};

const baseFindLocksInDomainRequest: object = {
  repositoryName: "",
  lockDomainId: "",
};

export const FindLocksInDomainRequest = {
  encode(
    message: FindLocksInDomainRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.lockDomainId !== "") {
      writer.uint32(18).string(message.lockDomainId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): FindLocksInDomainRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseFindLocksInDomainRequest,
    } as FindLocksInDomainRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.lockDomainId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindLocksInDomainRequest {
    const message = {
      ...baseFindLocksInDomainRequest,
    } as FindLocksInDomainRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.lockDomainId =
      object.lockDomainId !== undefined && object.lockDomainId !== null
        ? String(object.lockDomainId)
        : "";
    return message;
  },

  toJSON(message: FindLocksInDomainRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.lockDomainId !== undefined &&
      (obj.lockDomainId = message.lockDomainId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FindLocksInDomainRequest>, I>>(
    object: I
  ): FindLocksInDomainRequest {
    const message = {
      ...baseFindLocksInDomainRequest,
    } as FindLocksInDomainRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.lockDomainId = object.lockDomainId ?? "";
    return message;
  },
};

const baseFindLocksInDomainResponse: object = {};

export const FindLocksInDomainResponse = {
  encode(
    message: FindLocksInDomainResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    for (const v of message.locks) {
      Lock.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): FindLocksInDomainResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseFindLocksInDomainResponse,
    } as FindLocksInDomainResponse;
    message.locks = [];
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.locks.push(Lock.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): FindLocksInDomainResponse {
    const message = {
      ...baseFindLocksInDomainResponse,
    } as FindLocksInDomainResponse;
    message.locks = (object.locks ?? []).map((e: any) => Lock.fromJSON(e));
    return message;
  },

  toJSON(message: FindLocksInDomainResponse): unknown {
    const obj: any = {};
    if (message.locks) {
      obj.locks = message.locks.map((e) => (e ? Lock.toJSON(e) : undefined));
    } else {
      obj.locks = [];
    }
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<FindLocksInDomainResponse>, I>>(
    object: I
  ): FindLocksInDomainResponse {
    const message = {
      ...baseFindLocksInDomainResponse,
    } as FindLocksInDomainResponse;
    message.locks = object.locks?.map((e) => Lock.fromPartial(e)) || [];
    return message;
  },
};

const baseSaveTreeRequest: object = { repositoryName: "", hash: "" };

export const SaveTreeRequest = {
  encode(
    message: SaveTreeRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.tree !== undefined) {
      Tree.encode(message.tree, writer.uint32(18).fork()).ldelim();
    }
    if (message.hash !== "") {
      writer.uint32(26).string(message.hash);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SaveTreeRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseSaveTreeRequest } as SaveTreeRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.tree = Tree.decode(reader, reader.uint32());
          break;
        case 3:
          message.hash = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): SaveTreeRequest {
    const message = { ...baseSaveTreeRequest } as SaveTreeRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.tree =
      object.tree !== undefined && object.tree !== null
        ? Tree.fromJSON(object.tree)
        : undefined;
    message.hash =
      object.hash !== undefined && object.hash !== null
        ? String(object.hash)
        : "";
    return message;
  },

  toJSON(message: SaveTreeRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.tree !== undefined &&
      (obj.tree = message.tree ? Tree.toJSON(message.tree) : undefined);
    message.hash !== undefined && (obj.hash = message.hash);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<SaveTreeRequest>, I>>(
    object: I
  ): SaveTreeRequest {
    const message = { ...baseSaveTreeRequest } as SaveTreeRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.tree =
      object.tree !== undefined && object.tree !== null
        ? Tree.fromPartial(object.tree)
        : undefined;
    message.hash = object.hash ?? "";
    return message;
  },
};

const baseSaveTreeResponse: object = {};

export const SaveTreeResponse = {
  encode(
    _: SaveTreeResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): SaveTreeResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseSaveTreeResponse } as SaveTreeResponse;
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

  fromJSON(_: any): SaveTreeResponse {
    const message = { ...baseSaveTreeResponse } as SaveTreeResponse;
    return message;
  },

  toJSON(_: SaveTreeResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<SaveTreeResponse>, I>>(
    _: I
  ): SaveTreeResponse {
    const message = { ...baseSaveTreeResponse } as SaveTreeResponse;
    return message;
  },
};

const baseInsertCommitRequest: object = { repositoryName: "" };

export const InsertCommitRequest = {
  encode(
    message: InsertCommitRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.commit !== undefined) {
      Commit.encode(message.commit, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): InsertCommitRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInsertCommitRequest } as InsertCommitRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.commit = Commit.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InsertCommitRequest {
    const message = { ...baseInsertCommitRequest } as InsertCommitRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.commit =
      object.commit !== undefined && object.commit !== null
        ? Commit.fromJSON(object.commit)
        : undefined;
    return message;
  },

  toJSON(message: InsertCommitRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.commit !== undefined &&
      (obj.commit = message.commit ? Commit.toJSON(message.commit) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertCommitRequest>, I>>(
    object: I
  ): InsertCommitRequest {
    const message = { ...baseInsertCommitRequest } as InsertCommitRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.commit =
      object.commit !== undefined && object.commit !== null
        ? Commit.fromPartial(object.commit)
        : undefined;
    return message;
  },
};

const baseInsertCommitResponse: object = {};

export const InsertCommitResponse = {
  encode(
    _: InsertCommitResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): InsertCommitResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInsertCommitResponse } as InsertCommitResponse;
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

  fromJSON(_: any): InsertCommitResponse {
    const message = { ...baseInsertCommitResponse } as InsertCommitResponse;
    return message;
  },

  toJSON(_: InsertCommitResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertCommitResponse>, I>>(
    _: I
  ): InsertCommitResponse {
    const message = { ...baseInsertCommitResponse } as InsertCommitResponse;
    return message;
  },
};

const baseCommitToBranchRequest: object = { repositoryName: "" };

export const CommitToBranchRequest = {
  encode(
    message: CommitToBranchRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.commit !== undefined) {
      Commit.encode(message.commit, writer.uint32(18).fork()).ldelim();
    }
    if (message.branch !== undefined) {
      Branch.encode(message.branch, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CommitToBranchRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCommitToBranchRequest } as CommitToBranchRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.commit = Commit.decode(reader, reader.uint32());
          break;
        case 3:
          message.branch = Branch.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CommitToBranchRequest {
    const message = { ...baseCommitToBranchRequest } as CommitToBranchRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.commit =
      object.commit !== undefined && object.commit !== null
        ? Commit.fromJSON(object.commit)
        : undefined;
    message.branch =
      object.branch !== undefined && object.branch !== null
        ? Branch.fromJSON(object.branch)
        : undefined;
    return message;
  },

  toJSON(message: CommitToBranchRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.commit !== undefined &&
      (obj.commit = message.commit ? Commit.toJSON(message.commit) : undefined);
    message.branch !== undefined &&
      (obj.branch = message.branch ? Branch.toJSON(message.branch) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CommitToBranchRequest>, I>>(
    object: I
  ): CommitToBranchRequest {
    const message = { ...baseCommitToBranchRequest } as CommitToBranchRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.commit =
      object.commit !== undefined && object.commit !== null
        ? Commit.fromPartial(object.commit)
        : undefined;
    message.branch =
      object.branch !== undefined && object.branch !== null
        ? Branch.fromPartial(object.branch)
        : undefined;
    return message;
  },
};

const baseCommitToBranchResponse: object = {};

export const CommitToBranchResponse = {
  encode(
    _: CommitToBranchResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CommitToBranchResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCommitToBranchResponse } as CommitToBranchResponse;
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

  fromJSON(_: any): CommitToBranchResponse {
    const message = { ...baseCommitToBranchResponse } as CommitToBranchResponse;
    return message;
  },

  toJSON(_: CommitToBranchResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CommitToBranchResponse>, I>>(
    _: I
  ): CommitToBranchResponse {
    const message = { ...baseCommitToBranchResponse } as CommitToBranchResponse;
    return message;
  },
};

const baseCommitExistsRequest: object = { repositoryName: "", commitId: "" };

export const CommitExistsRequest = {
  encode(
    message: CommitExistsRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.commitId !== "") {
      writer.uint32(18).string(message.commitId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): CommitExistsRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCommitExistsRequest } as CommitExistsRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.commitId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CommitExistsRequest {
    const message = { ...baseCommitExistsRequest } as CommitExistsRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.commitId =
      object.commitId !== undefined && object.commitId !== null
        ? String(object.commitId)
        : "";
    return message;
  },

  toJSON(message: CommitExistsRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.commitId !== undefined && (obj.commitId = message.commitId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CommitExistsRequest>, I>>(
    object: I
  ): CommitExistsRequest {
    const message = { ...baseCommitExistsRequest } as CommitExistsRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.commitId = object.commitId ?? "";
    return message;
  },
};

const baseCommitExistsResponse: object = { exists: false };

export const CommitExistsResponse = {
  encode(
    message: CommitExistsResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.exists === true) {
      writer.uint32(8).bool(message.exists);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CommitExistsResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseCommitExistsResponse } as CommitExistsResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.exists = reader.bool();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CommitExistsResponse {
    const message = { ...baseCommitExistsResponse } as CommitExistsResponse;
    message.exists =
      object.exists !== undefined && object.exists !== null
        ? Boolean(object.exists)
        : false;
    return message;
  },

  toJSON(message: CommitExistsResponse): unknown {
    const obj: any = {};
    message.exists !== undefined && (obj.exists = message.exists);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CommitExistsResponse>, I>>(
    object: I
  ): CommitExistsResponse {
    const message = { ...baseCommitExistsResponse } as CommitExistsResponse;
    message.exists = object.exists ?? false;
    return message;
  },
};

const baseUpdateBranchRequest: object = { repositoryName: "" };

export const UpdateBranchRequest = {
  encode(
    message: UpdateBranchRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.branch !== undefined) {
      Branch.encode(message.branch, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UpdateBranchRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseUpdateBranchRequest } as UpdateBranchRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.branch = Branch.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): UpdateBranchRequest {
    const message = { ...baseUpdateBranchRequest } as UpdateBranchRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.branch =
      object.branch !== undefined && object.branch !== null
        ? Branch.fromJSON(object.branch)
        : undefined;
    return message;
  },

  toJSON(message: UpdateBranchRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.branch !== undefined &&
      (obj.branch = message.branch ? Branch.toJSON(message.branch) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<UpdateBranchRequest>, I>>(
    object: I
  ): UpdateBranchRequest {
    const message = { ...baseUpdateBranchRequest } as UpdateBranchRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.branch =
      object.branch !== undefined && object.branch !== null
        ? Branch.fromPartial(object.branch)
        : undefined;
    return message;
  },
};

const baseUpdateBranchResponse: object = {};

export const UpdateBranchResponse = {
  encode(
    _: UpdateBranchResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): UpdateBranchResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseUpdateBranchResponse } as UpdateBranchResponse;
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

  fromJSON(_: any): UpdateBranchResponse {
    const message = { ...baseUpdateBranchResponse } as UpdateBranchResponse;
    return message;
  },

  toJSON(_: UpdateBranchResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<UpdateBranchResponse>, I>>(
    _: I
  ): UpdateBranchResponse {
    const message = { ...baseUpdateBranchResponse } as UpdateBranchResponse;
    return message;
  },
};

const baseInsertBranchRequest: object = { repositoryName: "" };

export const InsertBranchRequest = {
  encode(
    message: InsertBranchRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.branch !== undefined) {
      Branch.encode(message.branch, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): InsertBranchRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInsertBranchRequest } as InsertBranchRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.branch = Branch.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): InsertBranchRequest {
    const message = { ...baseInsertBranchRequest } as InsertBranchRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.branch =
      object.branch !== undefined && object.branch !== null
        ? Branch.fromJSON(object.branch)
        : undefined;
    return message;
  },

  toJSON(message: InsertBranchRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.branch !== undefined &&
      (obj.branch = message.branch ? Branch.toJSON(message.branch) : undefined);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertBranchRequest>, I>>(
    object: I
  ): InsertBranchRequest {
    const message = { ...baseInsertBranchRequest } as InsertBranchRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.branch =
      object.branch !== undefined && object.branch !== null
        ? Branch.fromPartial(object.branch)
        : undefined;
    return message;
  },
};

const baseInsertBranchResponse: object = {};

export const InsertBranchResponse = {
  encode(
    _: InsertBranchResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): InsertBranchResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseInsertBranchResponse } as InsertBranchResponse;
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

  fromJSON(_: any): InsertBranchResponse {
    const message = { ...baseInsertBranchResponse } as InsertBranchResponse;
    return message;
  },

  toJSON(_: InsertBranchResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<InsertBranchResponse>, I>>(
    _: I
  ): InsertBranchResponse {
    const message = { ...baseInsertBranchResponse } as InsertBranchResponse;
    return message;
  },
};

const baseClearLockRequest: object = {
  repositoryName: "",
  lockDomainId: "",
  canonicalRelativePath: "",
};

export const ClearLockRequest = {
  encode(
    message: ClearLockRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.lockDomainId !== "") {
      writer.uint32(18).string(message.lockDomainId);
    }
    if (message.canonicalRelativePath !== "") {
      writer.uint32(26).string(message.canonicalRelativePath);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ClearLockRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseClearLockRequest } as ClearLockRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.lockDomainId = reader.string();
          break;
        case 3:
          message.canonicalRelativePath = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): ClearLockRequest {
    const message = { ...baseClearLockRequest } as ClearLockRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.lockDomainId =
      object.lockDomainId !== undefined && object.lockDomainId !== null
        ? String(object.lockDomainId)
        : "";
    message.canonicalRelativePath =
      object.canonicalRelativePath !== undefined &&
      object.canonicalRelativePath !== null
        ? String(object.canonicalRelativePath)
        : "";
    return message;
  },

  toJSON(message: ClearLockRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.lockDomainId !== undefined &&
      (obj.lockDomainId = message.lockDomainId);
    message.canonicalRelativePath !== undefined &&
      (obj.canonicalRelativePath = message.canonicalRelativePath);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ClearLockRequest>, I>>(
    object: I
  ): ClearLockRequest {
    const message = { ...baseClearLockRequest } as ClearLockRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.lockDomainId = object.lockDomainId ?? "";
    message.canonicalRelativePath = object.canonicalRelativePath ?? "";
    return message;
  },
};

const baseClearLockResponse: object = {};

export const ClearLockResponse = {
  encode(
    _: ClearLockResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ClearLockResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = { ...baseClearLockResponse } as ClearLockResponse;
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

  fromJSON(_: any): ClearLockResponse {
    const message = { ...baseClearLockResponse } as ClearLockResponse;
    return message;
  },

  toJSON(_: ClearLockResponse): unknown {
    const obj: any = {};
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<ClearLockResponse>, I>>(
    _: I
  ): ClearLockResponse {
    const message = { ...baseClearLockResponse } as ClearLockResponse;
    return message;
  },
};

const baseCountLocksInDomainRequest: object = {
  repositoryName: "",
  lockDomainId: "",
};

export const CountLocksInDomainRequest = {
  encode(
    message: CountLocksInDomainRequest,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.repositoryName !== "") {
      writer.uint32(10).string(message.repositoryName);
    }
    if (message.lockDomainId !== "") {
      writer.uint32(18).string(message.lockDomainId);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CountLocksInDomainRequest {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseCountLocksInDomainRequest,
    } as CountLocksInDomainRequest;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.repositoryName = reader.string();
          break;
        case 2:
          message.lockDomainId = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CountLocksInDomainRequest {
    const message = {
      ...baseCountLocksInDomainRequest,
    } as CountLocksInDomainRequest;
    message.repositoryName =
      object.repositoryName !== undefined && object.repositoryName !== null
        ? String(object.repositoryName)
        : "";
    message.lockDomainId =
      object.lockDomainId !== undefined && object.lockDomainId !== null
        ? String(object.lockDomainId)
        : "";
    return message;
  },

  toJSON(message: CountLocksInDomainRequest): unknown {
    const obj: any = {};
    message.repositoryName !== undefined &&
      (obj.repositoryName = message.repositoryName);
    message.lockDomainId !== undefined &&
      (obj.lockDomainId = message.lockDomainId);
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CountLocksInDomainRequest>, I>>(
    object: I
  ): CountLocksInDomainRequest {
    const message = {
      ...baseCountLocksInDomainRequest,
    } as CountLocksInDomainRequest;
    message.repositoryName = object.repositoryName ?? "";
    message.lockDomainId = object.lockDomainId ?? "";
    return message;
  },
};

const baseCountLocksInDomainResponse: object = { count: 0 };

export const CountLocksInDomainResponse = {
  encode(
    message: CountLocksInDomainResponse,
    writer: _m0.Writer = _m0.Writer.create()
  ): _m0.Writer {
    if (message.count !== 0) {
      writer.uint32(8).int32(message.count);
    }
    return writer;
  },

  decode(
    input: _m0.Reader | Uint8Array,
    length?: number
  ): CountLocksInDomainResponse {
    const reader = input instanceof _m0.Reader ? input : new _m0.Reader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = {
      ...baseCountLocksInDomainResponse,
    } as CountLocksInDomainResponse;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.count = reader.int32();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): CountLocksInDomainResponse {
    const message = {
      ...baseCountLocksInDomainResponse,
    } as CountLocksInDomainResponse;
    message.count =
      object.count !== undefined && object.count !== null
        ? Number(object.count)
        : 0;
    return message;
  },

  toJSON(message: CountLocksInDomainResponse): unknown {
    const obj: any = {};
    message.count !== undefined && (obj.count = Math.round(message.count));
    return obj;
  },

  fromPartial<I extends Exact<DeepPartial<CountLocksInDomainResponse>, I>>(
    object: I
  ): CountLocksInDomainResponse {
    const message = {
      ...baseCountLocksInDomainResponse,
    } as CountLocksInDomainResponse;
    message.count = object.count ?? 0;
    return message;
  },
};

export interface SourceControl {
  ping(request: DeepPartial<Empty>, metadata?: grpc.Metadata): Promise<Empty>;
  createRepository(
    request: DeepPartial<CreateRepositoryRequest>,
    metadata?: grpc.Metadata
  ): Promise<CreateRepositoryResponse>;
  destroyRepository(
    request: DeepPartial<DestroyRepositoryRequest>,
    metadata?: grpc.Metadata
  ): Promise<DestroyRepositoryResponse>;
  getBlobStorageUrl(
    request: DeepPartial<GetBlobStorageUrlRequest>,
    metadata?: grpc.Metadata
  ): Promise<GetBlobStorageUrlResponse>;
  insertWorkspace(
    request: DeepPartial<InsertWorkspaceRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertWorkspaceResponse>;
  findBranch(
    request: DeepPartial<FindBranchRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindBranchResponse>;
  readBranches(
    request: DeepPartial<ReadBranchesRequest>,
    metadata?: grpc.Metadata
  ): Promise<ReadBranchesResponse>;
  findBranchesInLockDomain(
    request: DeepPartial<FindBranchesInLockDomainRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindBranchesInLockDomainResponse>;
  readCommit(
    request: DeepPartial<ReadCommitRequest>,
    metadata?: grpc.Metadata
  ): Promise<ReadCommitResponse>;
  readTree(
    request: DeepPartial<ReadTreeRequest>,
    metadata?: grpc.Metadata
  ): Promise<ReadTreeResponse>;
  insertLock(
    request: DeepPartial<InsertLockRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertLockResponse>;
  findLock(
    request: DeepPartial<FindLockRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindLockResponse>;
  findLocksInDomain(
    request: DeepPartial<FindLocksInDomainRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindLocksInDomainResponse>;
  saveTree(
    request: DeepPartial<SaveTreeRequest>,
    metadata?: grpc.Metadata
  ): Promise<SaveTreeResponse>;
  insertCommit(
    request: DeepPartial<InsertCommitRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertCommitResponse>;
  commitToBranch(
    request: DeepPartial<CommitToBranchRequest>,
    metadata?: grpc.Metadata
  ): Promise<CommitToBranchResponse>;
  commitExists(
    request: DeepPartial<CommitExistsRequest>,
    metadata?: grpc.Metadata
  ): Promise<CommitExistsResponse>;
  updateBranch(
    request: DeepPartial<UpdateBranchRequest>,
    metadata?: grpc.Metadata
  ): Promise<UpdateBranchResponse>;
  insertBranch(
    request: DeepPartial<InsertBranchRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertBranchResponse>;
  clearLock(
    request: DeepPartial<ClearLockRequest>,
    metadata?: grpc.Metadata
  ): Promise<ClearLockResponse>;
  countLocksInDomain(
    request: DeepPartial<CountLocksInDomainRequest>,
    metadata?: grpc.Metadata
  ): Promise<CountLocksInDomainResponse>;
}

export class SourceControlClientImpl implements SourceControl {
  private readonly rpc: Rpc;

  constructor(rpc: Rpc) {
    this.rpc = rpc;
    this.ping = this.ping.bind(this);
    this.createRepository = this.createRepository.bind(this);
    this.destroyRepository = this.destroyRepository.bind(this);
    this.getBlobStorageUrl = this.getBlobStorageUrl.bind(this);
    this.insertWorkspace = this.insertWorkspace.bind(this);
    this.findBranch = this.findBranch.bind(this);
    this.readBranches = this.readBranches.bind(this);
    this.findBranchesInLockDomain = this.findBranchesInLockDomain.bind(this);
    this.readCommit = this.readCommit.bind(this);
    this.readTree = this.readTree.bind(this);
    this.insertLock = this.insertLock.bind(this);
    this.findLock = this.findLock.bind(this);
    this.findLocksInDomain = this.findLocksInDomain.bind(this);
    this.saveTree = this.saveTree.bind(this);
    this.insertCommit = this.insertCommit.bind(this);
    this.commitToBranch = this.commitToBranch.bind(this);
    this.commitExists = this.commitExists.bind(this);
    this.updateBranch = this.updateBranch.bind(this);
    this.insertBranch = this.insertBranch.bind(this);
    this.clearLock = this.clearLock.bind(this);
    this.countLocksInDomain = this.countLocksInDomain.bind(this);
  }

  ping(request: DeepPartial<Empty>, metadata?: grpc.Metadata): Promise<Empty> {
    return this.rpc.unary(
      SourceControlPingDesc,
      Empty.fromPartial(request),
      metadata
    );
  }

  createRepository(
    request: DeepPartial<CreateRepositoryRequest>,
    metadata?: grpc.Metadata
  ): Promise<CreateRepositoryResponse> {
    return this.rpc.unary(
      SourceControlCreateRepositoryDesc,
      CreateRepositoryRequest.fromPartial(request),
      metadata
    );
  }

  destroyRepository(
    request: DeepPartial<DestroyRepositoryRequest>,
    metadata?: grpc.Metadata
  ): Promise<DestroyRepositoryResponse> {
    return this.rpc.unary(
      SourceControlDestroyRepositoryDesc,
      DestroyRepositoryRequest.fromPartial(request),
      metadata
    );
  }

  getBlobStorageUrl(
    request: DeepPartial<GetBlobStorageUrlRequest>,
    metadata?: grpc.Metadata
  ): Promise<GetBlobStorageUrlResponse> {
    return this.rpc.unary(
      SourceControlGetBlobStorageUrlDesc,
      GetBlobStorageUrlRequest.fromPartial(request),
      metadata
    );
  }

  insertWorkspace(
    request: DeepPartial<InsertWorkspaceRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertWorkspaceResponse> {
    return this.rpc.unary(
      SourceControlInsertWorkspaceDesc,
      InsertWorkspaceRequest.fromPartial(request),
      metadata
    );
  }

  findBranch(
    request: DeepPartial<FindBranchRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindBranchResponse> {
    return this.rpc.unary(
      SourceControlFindBranchDesc,
      FindBranchRequest.fromPartial(request),
      metadata
    );
  }

  readBranches(
    request: DeepPartial<ReadBranchesRequest>,
    metadata?: grpc.Metadata
  ): Promise<ReadBranchesResponse> {
    return this.rpc.unary(
      SourceControlReadBranchesDesc,
      ReadBranchesRequest.fromPartial(request),
      metadata
    );
  }

  findBranchesInLockDomain(
    request: DeepPartial<FindBranchesInLockDomainRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindBranchesInLockDomainResponse> {
    return this.rpc.unary(
      SourceControlFindBranchesInLockDomainDesc,
      FindBranchesInLockDomainRequest.fromPartial(request),
      metadata
    );
  }

  readCommit(
    request: DeepPartial<ReadCommitRequest>,
    metadata?: grpc.Metadata
  ): Promise<ReadCommitResponse> {
    return this.rpc.unary(
      SourceControlReadCommitDesc,
      ReadCommitRequest.fromPartial(request),
      metadata
    );
  }

  readTree(
    request: DeepPartial<ReadTreeRequest>,
    metadata?: grpc.Metadata
  ): Promise<ReadTreeResponse> {
    return this.rpc.unary(
      SourceControlReadTreeDesc,
      ReadTreeRequest.fromPartial(request),
      metadata
    );
  }

  insertLock(
    request: DeepPartial<InsertLockRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertLockResponse> {
    return this.rpc.unary(
      SourceControlInsertLockDesc,
      InsertLockRequest.fromPartial(request),
      metadata
    );
  }

  findLock(
    request: DeepPartial<FindLockRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindLockResponse> {
    return this.rpc.unary(
      SourceControlFindLockDesc,
      FindLockRequest.fromPartial(request),
      metadata
    );
  }

  findLocksInDomain(
    request: DeepPartial<FindLocksInDomainRequest>,
    metadata?: grpc.Metadata
  ): Promise<FindLocksInDomainResponse> {
    return this.rpc.unary(
      SourceControlFindLocksInDomainDesc,
      FindLocksInDomainRequest.fromPartial(request),
      metadata
    );
  }

  saveTree(
    request: DeepPartial<SaveTreeRequest>,
    metadata?: grpc.Metadata
  ): Promise<SaveTreeResponse> {
    return this.rpc.unary(
      SourceControlSaveTreeDesc,
      SaveTreeRequest.fromPartial(request),
      metadata
    );
  }

  insertCommit(
    request: DeepPartial<InsertCommitRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertCommitResponse> {
    return this.rpc.unary(
      SourceControlInsertCommitDesc,
      InsertCommitRequest.fromPartial(request),
      metadata
    );
  }

  commitToBranch(
    request: DeepPartial<CommitToBranchRequest>,
    metadata?: grpc.Metadata
  ): Promise<CommitToBranchResponse> {
    return this.rpc.unary(
      SourceControlCommitToBranchDesc,
      CommitToBranchRequest.fromPartial(request),
      metadata
    );
  }

  commitExists(
    request: DeepPartial<CommitExistsRequest>,
    metadata?: grpc.Metadata
  ): Promise<CommitExistsResponse> {
    return this.rpc.unary(
      SourceControlCommitExistsDesc,
      CommitExistsRequest.fromPartial(request),
      metadata
    );
  }

  updateBranch(
    request: DeepPartial<UpdateBranchRequest>,
    metadata?: grpc.Metadata
  ): Promise<UpdateBranchResponse> {
    return this.rpc.unary(
      SourceControlUpdateBranchDesc,
      UpdateBranchRequest.fromPartial(request),
      metadata
    );
  }

  insertBranch(
    request: DeepPartial<InsertBranchRequest>,
    metadata?: grpc.Metadata
  ): Promise<InsertBranchResponse> {
    return this.rpc.unary(
      SourceControlInsertBranchDesc,
      InsertBranchRequest.fromPartial(request),
      metadata
    );
  }

  clearLock(
    request: DeepPartial<ClearLockRequest>,
    metadata?: grpc.Metadata
  ): Promise<ClearLockResponse> {
    return this.rpc.unary(
      SourceControlClearLockDesc,
      ClearLockRequest.fromPartial(request),
      metadata
    );
  }

  countLocksInDomain(
    request: DeepPartial<CountLocksInDomainRequest>,
    metadata?: grpc.Metadata
  ): Promise<CountLocksInDomainResponse> {
    return this.rpc.unary(
      SourceControlCountLocksInDomainDesc,
      CountLocksInDomainRequest.fromPartial(request),
      metadata
    );
  }
}

export const SourceControlDesc = {
  serviceName: "source_control.SourceControl",
};

export const SourceControlPingDesc: UnaryMethodDefinitionish = {
  methodName: "Ping",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return Empty.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...Empty.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlCreateRepositoryDesc: UnaryMethodDefinitionish = {
  methodName: "CreateRepository",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return CreateRepositoryRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...CreateRepositoryResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlDestroyRepositoryDesc: UnaryMethodDefinitionish = {
  methodName: "DestroyRepository",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return DestroyRepositoryRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...DestroyRepositoryResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlGetBlobStorageUrlDesc: UnaryMethodDefinitionish = {
  methodName: "GetBlobStorageUrl",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return GetBlobStorageUrlRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...GetBlobStorageUrlResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlInsertWorkspaceDesc: UnaryMethodDefinitionish = {
  methodName: "InsertWorkspace",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return InsertWorkspaceRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...InsertWorkspaceResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlFindBranchDesc: UnaryMethodDefinitionish = {
  methodName: "FindBranch",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return FindBranchRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...FindBranchResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlReadBranchesDesc: UnaryMethodDefinitionish = {
  methodName: "ReadBranches",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return ReadBranchesRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...ReadBranchesResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlFindBranchesInLockDomainDesc: UnaryMethodDefinitionish =
  {
    methodName: "FindBranchesInLockDomain",
    service: SourceControlDesc,
    requestStream: false,
    responseStream: false,
    requestType: {
      serializeBinary() {
        return FindBranchesInLockDomainRequest.encode(this).finish();
      },
    } as any,
    responseType: {
      deserializeBinary(data: Uint8Array) {
        return {
          ...FindBranchesInLockDomainResponse.decode(data),
          toObject() {
            return this;
          },
        };
      },
    } as any,
  };

export const SourceControlReadCommitDesc: UnaryMethodDefinitionish = {
  methodName: "ReadCommit",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return ReadCommitRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...ReadCommitResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlReadTreeDesc: UnaryMethodDefinitionish = {
  methodName: "ReadTree",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return ReadTreeRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...ReadTreeResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlInsertLockDesc: UnaryMethodDefinitionish = {
  methodName: "InsertLock",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return InsertLockRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...InsertLockResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlFindLockDesc: UnaryMethodDefinitionish = {
  methodName: "FindLock",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return FindLockRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...FindLockResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlFindLocksInDomainDesc: UnaryMethodDefinitionish = {
  methodName: "FindLocksInDomain",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return FindLocksInDomainRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...FindLocksInDomainResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlSaveTreeDesc: UnaryMethodDefinitionish = {
  methodName: "SaveTree",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return SaveTreeRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...SaveTreeResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlInsertCommitDesc: UnaryMethodDefinitionish = {
  methodName: "InsertCommit",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return InsertCommitRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...InsertCommitResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlCommitToBranchDesc: UnaryMethodDefinitionish = {
  methodName: "CommitToBranch",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return CommitToBranchRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...CommitToBranchResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlCommitExistsDesc: UnaryMethodDefinitionish = {
  methodName: "CommitExists",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return CommitExistsRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...CommitExistsResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlUpdateBranchDesc: UnaryMethodDefinitionish = {
  methodName: "UpdateBranch",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return UpdateBranchRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...UpdateBranchResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlInsertBranchDesc: UnaryMethodDefinitionish = {
  methodName: "InsertBranch",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return InsertBranchRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...InsertBranchResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlClearLockDesc: UnaryMethodDefinitionish = {
  methodName: "ClearLock",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return ClearLockRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...ClearLockResponse.decode(data),
        toObject() {
          return this;
        },
      };
    },
  } as any,
};

export const SourceControlCountLocksInDomainDesc: UnaryMethodDefinitionish = {
  methodName: "CountLocksInDomain",
  service: SourceControlDesc,
  requestStream: false,
  responseStream: false,
  requestType: {
    serializeBinary() {
      return CountLocksInDomainRequest.encode(this).finish();
    },
  } as any,
  responseType: {
    deserializeBinary(data: Uint8Array) {
      return {
        ...CountLocksInDomainResponse.decode(data),
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

function toTimestamp(date: Date): Timestamp {
  const seconds = date.getTime() / 1_000;
  const nanos = (date.getTime() % 1_000) * 1_000_000;
  return { seconds, nanos };
}

function fromTimestamp(t: Timestamp): Date {
  let millis = t.seconds * 1_000;
  millis += t.nanos / 1_000_000;
  return new Date(millis);
}

function fromJsonTimestamp(o: any): Date {
  if (o instanceof Date) {
    return o;
  } else if (typeof o === "string") {
    return new Date(o);
  } else {
    return fromTimestamp(Timestamp.fromJSON(o));
  }
}

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}
