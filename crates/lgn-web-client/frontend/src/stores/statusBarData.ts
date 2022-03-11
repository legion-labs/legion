import { writable } from "svelte/store";

export const statusStore = writable<string | null>(null);
