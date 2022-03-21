import type { SpanTrack } from "@lgn/proto-telemetry/dist/span";
import type { BlockMetadata } from "@lgn/proto-telemetry/dist/block";
import type { LODState } from "./LodState";

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
