import type { CumulativeCallGraphComputedBlock } from "@lgn/proto-telemetry/dist/callgraph";

import { CallGraphState } from "./CallGraphState";
import { CallGraphThread } from "./CallGraphThread";

export class CallGraphHierarchyState extends CallGraphState {
  threads: Map<number, CallGraphThread> = new Map();

  clear(): void {
    this.threads = new Map();
  }

  ingestBlockImpl(block: CumulativeCallGraphComputedBlock): void {
    let thread = this.threads.get(block.streamHash);
    if (!thread) {
      thread = new CallGraphThread(block);
      this.threads.set(block.streamHash, thread);
    } else {
      thread.ingestBlock(block);
    }
  }
}
