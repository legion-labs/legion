import { writable } from "svelte/store";

import type { SourceControl } from "@lgn/apis/editor";

export const selectedLocalChange =
  writable<SourceControl.StagedResource | null>(null);
