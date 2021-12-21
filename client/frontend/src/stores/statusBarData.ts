import { writable, Writable } from "svelte/store";

export const statusStore: Writable<string | null> = writable(null);
