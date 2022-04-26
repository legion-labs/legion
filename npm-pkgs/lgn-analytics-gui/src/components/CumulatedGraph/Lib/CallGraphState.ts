import type { CumulativeCallGraphComputedBlock } from "@lgn/proto-telemetry/dist/callgraph";
import type { ScopeDesc } from "@lgn/proto-telemetry/dist/calltree";
import { CallGraphThread } from "./CallGraphThread";

export class CallGraphState {
  startTicks: number;
  tscFrequency: number;
  begin: number;
  end: number;
  scopes: Record<number, ScopeDesc> = {};
  threads: Map<number, CallGraphThread> = new Map();
  cache: Map<string, CumulativeCallGraphComputedBlock> = new Map();
  constructor(
    startTicks: number,
    tscFrequency: number,
    begin: number,
    end: number
  ) {
    this.startTicks = startTicks;
    this.tscFrequency = tscFrequency;
    this.begin = begin;
    this.end = end;
  }

  ingestBlock(blockId: string, block: CumulativeCallGraphComputedBlock) {
    this.scopes = { ...this.scopes, ...block.scopes };
    if (block.full) {
      this.cache.set(blockId, block);
    }
    let thread = this.threads.get(block.streamHash);
    if (!thread) {
      this.scopes[block.streamHash] = {
        name: `Thread: ${block.streamName}`,
        hash: block.streamHash,
        filename: "",
        line: 0,
      };
      thread = new CallGraphThread(block);
      this.threads.set(block.streamHash, thread);
    } else {
      thread.ingestBlock(block);
    }
  }
}
