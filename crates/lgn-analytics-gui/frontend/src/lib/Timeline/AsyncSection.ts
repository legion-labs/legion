import { LODState } from "./LodState";
import { SpanTrack } from "@lgn/proto-telemetry/dist/analytics";

export type AsyncSection = {
  sectionSequenceNumber: number;
  sectionLod: number;
  state: LODState;
  tracks: SpanTrack[];
};
