import { BlockAsyncEventsStatReply } from "@lgn/proto-telemetry/dist/analytics";
import { AsyncSection } from "./AsyncSection";

// ProcessAsyncData contains the data about the async tasks of one process
export type ProcessAsyncData = {
  processId: string;
  maxDepth: number;
  minMs: number;
  maxMs: number;
  blockStats: BlockAsyncEventsStatReply[];
  sections: AsyncSection[];
};
