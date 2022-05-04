import { writable } from "svelte/store";

import type { CumulativeCallGraphBlockDesc } from "@lgn/proto-telemetry/dist/callgraph";
import { displayError } from "@lgn/web-client/src/lib/errors";
import log from "@lgn/web-client/src/lib/log";

import { makeGrpcClient } from "@/lib/client";

import type { LoadingStore } from "../Misc/LoadingStore";
import { CallGraphFlatState } from "./CallGraphFlatState";
import { CallGraphHierarchyState } from "./CallGraphHierarchyState";
import type { CallGraphState } from "./CallGraphState";

export type CumulatedCallGraphFlatStore = Awaited<
  ReturnType<typeof getProcessCumulatedCallGraphFlat>
>;

export type CumulatedCallGraphHierarchyStore = Awaited<
  ReturnType<typeof getProcessCumulatedCallGraphHierarchy>
>;

export async function getProcessCumulatedCallGraphHierarchy(
  processId: string,
  begin: number,
  end: number,
  loadingStore: LoadingStore | null = null
) {
  return getProcessCumulatedCallGraph<CallGraphHierarchyState>(
    processId,
    begin,
    end,
    new CallGraphHierarchyState(),
    loadingStore
  );
}

export async function getProcessCumulatedCallGraphFlat(
  processId: string,
  begin: number,
  end: number,
  loadingStore: LoadingStore | null = null
) {
  return getProcessCumulatedCallGraph<CallGraphFlatState>(
    processId,
    begin,
    end,
    new CallGraphFlatState(),
    loadingStore
  );
}

async function getProcessCumulatedCallGraph<T extends CallGraphState>(
  processId: string,
  begin: number,
  end: number,
  state: T,
  loadingStore: LoadingStore | null = null
) {
  const { subscribe, set, update } = writable<T>();

  const client = makeGrpcClient();

  set(state);

  const updateState = (action: (state: T) => void) => {
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

    if (!state.loading) {
      updateState((s) => {
        s.loading = true;
      });
    }

    if (loadingStore) {
      loadingStore.addWork();
    }

    const result = await client
      .fetch_cumulative_call_graph_computed_block({
        blockId: blockDesc.id,
        tscFrequency: state.tscFrequency,
        startTicks: state.startTicks,
        beginMs: state.begin,
        endMs: state.end,
      })
      .finally(() => {
        if (loadingStore) {
          loadingStore.completeWork();
        }
      });

    return result;
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
                if (state.loading) {
                  state.loading = false;
                }
              }
            })
          )
      );
    });

    if (promises.length) {
      await Promise.any(promises).catch((e) => {
        if (e instanceof AggregateError) {
          for (const error of e.errors) {
            log.error(displayError(error));
          }
        } else {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
          log.debug("call-graph", e);
        }
      });
    }
  };

  await updateRange(begin, end);

  return { subscribe, updateRange };
}
