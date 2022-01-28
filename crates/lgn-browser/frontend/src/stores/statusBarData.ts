import { Writable } from "../lib/store";

export const statusStore = new Writable<string | null>(null);
