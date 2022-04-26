import { makeGrpcClient } from "@/lib/client";
import type { CumulativeCallGraphBlockDesc } from "@lgn/proto-telemetry/dist/callgraph";
import { writable } from "svelte/store";
import { CallGraphState } from "./CallGraphState";

export type CumulatedCallGraphStore = Awaited<
  ReturnType<typeof getProcessCumulatedCallGraph>
>;

export async function getProcessCumulatedCallGraph(
  processId: string,
  begin: number,
  end: number
) {
  const { subscribe, set, update } = writable<CallGraphState>();

  const client = makeGrpcClient();

  const state = new CallGraphState();

  set(state);

  const updateState = (action: (state: CallGraphState) => void) => {
    update((s) => {
      action(s);
      return s;
    });
  };

  const fetchBlock = async (
    state: CallGraphState,
    blockDesc: CumulativeCallGraphBlockDesc
  ) => {
    if (blockDesc.full) {
      const block = state.cache.get(blockDesc.id);
      if (block) {
        return block;
      }
    }
    return await client.fetch_cumulative_call_graph_computed_block({
      blockId: blockDesc.id,
      tscFrequency: state.tscFrequency,
      startTicks: state.startTicks,
      beginMs: state.begin,
      endMs: state.end,
    });
  };

  const updateRange = async (begin: number, end: number) => {
    const { blocks, startTicks, tscFrequency } =
      await client.fetch_cumulative_call_graph_manifest({
        processId: processId,
        beginMs: begin,
        endMs: end,
      });

    updateState((state) => {
      state.setNewParameters(startTicks, tscFrequency, begin, end);
    });

    const promises: Promise<void>[] = [];

    blocks.forEach((desc) => {
      promises.push(
        fetchBlock(state, desc)
          .catch((e) => console.error(e))
          .then((b) =>
            updateState((state) => {
              if (b) {
                state.ingestBlock(desc.id, b);
              }
            })
          )
      );
    });

    await Promise.any(promises).catch((e) => console.error(e));

    updateState((state) => {
      state.loading = false;
    });
  };

  await updateRange(begin, end);

  return { subscribe, updateRange };
}
