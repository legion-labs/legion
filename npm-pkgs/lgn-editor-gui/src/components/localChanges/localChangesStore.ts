import { writable } from "svelte/store";

import type { StagedResource } from "@lgn/proto-editor/dist/source_control";

export const selectedLocalChange = writable<StagedResource | null>(null);
