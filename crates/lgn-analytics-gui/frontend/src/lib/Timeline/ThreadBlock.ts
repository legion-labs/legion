import { SpanTrack } from "@lgn/proto-telemetry/dist/analytics";
import { BlockMetadata } from "@lgn/proto-telemetry/dist/block";

export type ThreadBlock = {
  blockDefinition: BlockMetadata; // block metadata stored in data lake
  beginMs: number; // relative to main process
  endMs: number; // relative to main process
  lods: ThreadBlockLOD[];
};

export type ThreadBlockLOD = {
  state: LODState;
  tracks: SpanTrack[];
  lodId: number;
};

export enum LODState {
  Missing,
  Requested,
  Loaded,
}
