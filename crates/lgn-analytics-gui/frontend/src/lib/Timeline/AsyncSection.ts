import type { LODState } from "./LodState";
import type { SpanTrack } from "@lgn/proto-telemetry/dist/span";

export type AsyncSection = {
  sectionSequenceNumber: number;
  sectionLod: number;
  state: LODState;
  tracks: SpanTrack[];
};
