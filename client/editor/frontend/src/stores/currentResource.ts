import { writable } from "svelte/store";
import { ResourceWithProperties } from "@/api/propertyGrid";

export default writable<ResourceWithProperties | null>(null);
