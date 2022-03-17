import type { FluentBundle } from "@fluent/bundle";
import { derived, Readable } from "svelte/store";
import type { MapStore, MapValue } from "./map";
import { createMapStore } from "./map";

export type BundlesValue = MapValue<string, FluentBundle>;

export type BundlesStore = MapStore<string, FluentBundle>;

export function createBundlesStore(
  initialBundles?: Map<string, FluentBundle>
): BundlesStore {
  return createMapStore(initialBundles);
}

export type AvailableLocalesValue = string[];

export type AvailableLocalesStore = Readable<AvailableLocalesValue>;

export function createAvailableLocalesStore(
  bundles: BundlesStore
): AvailableLocalesStore {
  return derived(bundles, ($bundles) => Array.from($bundles.keys()));
}
