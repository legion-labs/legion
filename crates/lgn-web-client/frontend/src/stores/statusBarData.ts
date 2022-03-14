import type { Writable } from "svelte/store";
import { writable } from "svelte/store";

export type StatusBarDataValue = string | null;

export type StatusBarDataStore = Writable<StatusBarDataValue>;

export default writable<StatusBarDataValue>(null);
