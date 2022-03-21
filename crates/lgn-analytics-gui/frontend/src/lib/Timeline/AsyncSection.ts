import type { LODState } from "./LodState";
import type { SpanTrack } from "@lgn/proto-telemetry/dist/analytics";

export type AsyncSection = {
  sectionSequenceNumber: number;
  sectionLod: number;
  state: LODState;
  tracks: SpanTrack[];
};
