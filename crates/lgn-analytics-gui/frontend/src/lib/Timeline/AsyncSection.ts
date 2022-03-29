import type { SpanTrack } from "@lgn/proto-telemetry/dist/span";

import type { LODState } from "./LodState";

export type AsyncSection = {
  sectionSequenceNumber: number;
  sectionLod: number;
  state: LODState;
  tracks: SpanTrack[];
};
