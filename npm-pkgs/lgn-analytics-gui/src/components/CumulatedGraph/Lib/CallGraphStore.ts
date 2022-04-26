import { makeGrpcClient } from "@/lib/client";
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

  const fetchBlock = async (state: CallGraphState, blockId: string) => {
    // caching !
    const block = await client.fetch_cumulative_call_graph_computed_block({
      blockId: blockId,
      tscFrequency: state.tscFrequency,
      startTicks: state.startTicks,
      beginMs: state.begin,
      endMs: state.end,
    });
    update((s) => {
      s.ingestBlock(blockId, block);
      return s;
    });
  };

  const updateRange = async (begin: number, end: number) => {
    const { blocks, startTicks, tscFrequency } =
      await client.fetch_cumulative_call_graph_manifest({
        processId: processId,
        beginMs: begin,
        endMs: end,
      });

    const state = new CallGraphState(startTicks, tscFrequency, begin, end);

    set(state);

    const promises: Promise<void>[] = [];

    blocks.forEach((id) => {
      promises.push(fetchBlock(state, id).catch((e) => console.error(e)));
    });

    await Promise.any(promises);
  };

  await updateRange(begin, end);

  return { subscribe, updateRange };
}
