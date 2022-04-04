import { writable } from "svelte/store";

import type { ScopeDesc } from "@lgn/proto-telemetry/dist/calltree";

import { GraphNodeState } from "./GraphNodeState";

export const scopeStore = writable<Record<number, ScopeDesc>>({});
export const graphStateStore = writable<GraphNodeState>(new GraphNodeState());
