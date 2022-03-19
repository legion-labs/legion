import type { Writable } from "svelte/store";
import { writable } from "svelte/store";
import type { StagedResource } from "@lgn/proto-editor/dist/source_control";
import { getStagedResources } from "@/api";

export type StagedResourcesValue = StagedResource[] | null;

export type StagedResourcesStore = Writable<StagedResourcesValue>;

export const stagedResources: StagedResourcesStore = writable(null);

export async function initStagedResourcesStream(pollInternal = 2_000) {
  const { entries } = await getStagedResources();

  stagedResources.set(entries);

  // TODO: Have a stream on the backend?
  const intervalId = setInterval(async () => {
    const { entries } = await getStagedResources();

    stagedResources.set(entries);
  }, pollInternal);

  return () => clearInterval(intervalId);
}
