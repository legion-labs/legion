import { writable } from "svelte/store";
import { ResourceWithProperties } from "@/api";

export default writable<ResourceWithProperties | null>(null);
