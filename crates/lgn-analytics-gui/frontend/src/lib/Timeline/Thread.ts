import { Stream } from "@lgn/proto-telemetry/dist/stream";

export type Thread = {
  streamInfo: Stream;
  maxDepth: number;
  minMs: number;
  maxMs: number;
  block_ids: string[];
};
