import { writable } from "svelte/store";

import type { SourceControl } from "@lgn/api/editor";

export const selectedLocalChange =
  writable<SourceControl.StagedResource | null>(null);
