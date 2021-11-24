import { writable, Writable } from "svelte/store";

export const statusStore: Writable<String | null> = writable(null);
