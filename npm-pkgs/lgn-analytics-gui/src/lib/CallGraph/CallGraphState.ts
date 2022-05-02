import type { CumulativeCallGraphComputedBlock } from "@lgn/proto-telemetry/dist/callgraph";
import type { ScopeDesc } from "@lgn/proto-telemetry/dist/calltree";

export abstract class CallGraphState {
  startTicks = NaN;
  tscFrequency = NaN;
  begin = NaN;
  end = NaN;
  scopes: Record<number, ScopeDesc> = {};
  cache: Map<string, CumulativeCallGraphComputedBlock> = new Map();
  loading = true;
  setNewParameters(
    startTicks: number,
    tscFrequency: number,
    begin: number,
    end: number
  ) {
    this.startTicks = startTicks;
    this.tscFrequency = tscFrequency;
    this.begin = begin;
    this.end = end;
    this.clear();
  }

  abstract clear(): void;

  abstract ingestBlockImpl(block: CumulativeCallGraphComputedBlock): void;

  ingestBlock(blockId: string, block: CumulativeCallGraphComputedBlock) {
    this.scopes = { ...this.scopes, ...block.scopes };
    this.scopes[block.streamHash] = {
      name: `Thread: ${block.streamName}`,
      hash: block.streamHash,
      filename: "",
      line: 0,
    };
    if (block.full && !this.cache.has(blockId)) {
      this.cache.set(blockId, block);
    }
    this.ingestBlockImpl(block);
  }
}
