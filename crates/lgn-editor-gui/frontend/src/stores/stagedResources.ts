import type { Writable } from "svelte/store";
import { get, writable } from "svelte/store";
import type { StagedResource } from "@lgn/proto-editor/dist/source_control";
import {
  commitStagedResources,
  getAllResources,
  getStagedResources,
  syncLatest,
} from "@/api";
import allResources from "./allResources";
import log from "@lgn/web-client/src/lib/log";

export type StagedResourcesValue = StagedResource[] | null;

export type StagedResourcesStore = Writable<StagedResourcesValue>;

export const stagedResources: StagedResourcesStore = writable(null);

export async function initStagedResourcesStream(pollInternal = 2_000) {
  const { entries } = await getStagedResources();

  stagedResources.set(entries);

  // TODO: Have a stream on the backend?
  const intervalId = setInterval(() => {
    getStagedResources()
      .then(({ entries }) => stagedResources.set(entries))
      // TODO: Handle errors
      .catch(() => undefined);
  }, pollInternal);

  return () => clearInterval(intervalId);
}

export function syncFromMain() {
  return Promise.all([syncLatest(), allResources.run(getAllResources)]);
}

export async function submitToMain(message: string) {
  const resources = get(stagedResources);

  if (!resources?.length) {
    log.debug("No local changes to commit");

    return;
  }

  log.debug(
    log.json`Committing the following resources ${get(stagedResources)}`
  );

  await commitStagedResources({ message });
}

export type StagedResourcesModeValue = "card" | "list";

export type StagedResourcesModeStore = Writable<StagedResourcesModeValue>;

export const stagedResourcesMode: StagedResourcesModeStore = writable("card");
