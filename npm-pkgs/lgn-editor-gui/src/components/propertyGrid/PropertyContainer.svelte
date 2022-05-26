<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { Writable } from "svelte/store";

  import type { PropertyUpdate } from "@/api";
  import { propertyIsBag } from "@/components/propertyGrid/lib/propertyGrid";
  import type {
    BagResourceProperty,
    ResourceProperty,
  } from "@/components/propertyGrid/lib/propertyGrid";
  import type { PropertyGridStore } from "@/stores/propertyGrid";

  import PropertyBag from "./PropertyBag.svelte";
  import PropertyUnit from "./PropertyUnit.svelte";
  import type {
    AddVectorSubPropertyEvent,
    RemoveVectorSubPropertyEvent,
  } from "./types";

  const dispatch = createEventDispatcher<{
    input: PropertyUpdate;
    addVectorSubProperty: AddVectorSubPropertyEvent;
    removeVectorSubProperty: RemoveVectorSubPropertyEvent;
    displayable: boolean;
  }>();

  export let propertyGridStore: PropertyGridStore;

  export let property: ResourceProperty;

  export let parentProperty: BagResourceProperty | null = null;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];

  /** The property index (only used in vectors) */
  export let index: number;

  export let search: Writable<string>;
</script>

<div>
  {#if propertyIsBag(property)}
    <PropertyBag
      on:displayable
      on:input={(event) => dispatch("input", event.detail)}
      on:addVectorSubProperty={(event) =>
        dispatch("addVectorSubProperty", event.detail)}
      on:removeVectorSubProperty={(event) =>
        dispatch("removeVectorSubProperty", event.detail)}
      bind:parentProperty
      {property}
      {level}
      {search}
      {pathParts}
      {propertyGridStore}
    />
  {:else}
    <PropertyUnit
      on:displayable
      on:input={(event) => dispatch("input", event.detail)}
      on:removeVectorSubProperty={(event) =>
        dispatch("removeVectorSubProperty", event.detail)}
      {property}
      bind:parentProperty
      {pathParts}
      {search}
      {index}
      {level}
    />
  {/if}
</div>
