<script lang="ts">
  import { PropertyUpdate } from "@/api";

  import {
    BagResourceProperty,
    propertyIsBag,
    ResourceProperty,
  } from "@/lib/propertyGrid";
  import { createEventDispatcher } from "svelte";
  import PropertyBag from "./PropertyBag.svelte";
  import PropertyUnit from "./PropertyUnit.svelte";
  import {
    AddVectorSubPropertyEvent,
    RemoveVectorSubPropertyEvent,
  } from "./types";

  const dispatch = createEventDispatcher<{
    input: PropertyUpdate;
    addVectorSubProperty: AddVectorSubPropertyEvent;
    removeVectorSubProperty: RemoveVectorSubPropertyEvent;
  }>();

  export let property: ResourceProperty;

  export let parentProperty: BagResourceProperty | null = null;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];
</script>

<div class="root">
  {#if propertyIsBag(property)}
    <PropertyBag
      on:input={(event) => dispatch("input", event.detail)}
      on:addVectorSubProperty={(event) =>
        dispatch("addVectorSubProperty", event.detail)}
      on:removeVectorSubProperty={(event) =>
        dispatch("removeVectorSubProperty", event.detail)}
      {property}
      {level}
      {pathParts}
    />
  {:else}
    <PropertyUnit
      on:input={(event) => dispatch("input", event.detail)}
      on:removeVectorSubProperty={(event) =>
        dispatch("removeVectorSubProperty", event.detail)}
      {property}
      bind:parentProperty
      {pathParts}
    />
  {/if}
</div>
