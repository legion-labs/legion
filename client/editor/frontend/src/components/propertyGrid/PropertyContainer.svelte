<script lang="ts">
  import {
    BagResourceProperty,
    propertyIsBag,
    ResourceProperty,
  } from "@/lib/propertyGrid";
  import PropertyBag from "./PropertyBag.svelte";
  import PropertyUnit from "./PropertyUnit.svelte";

  export let property: ResourceProperty;

  export let parentProperty: BagResourceProperty | null = null;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];
</script>

<div class="root">
  {#if propertyIsBag(property)}
    <PropertyBag on:input {property} {level} {pathParts} />
  {:else}
    <PropertyUnit
      on:input
      on:removeVectorProperty
      {property}
      {parentProperty}
      {pathParts}
    />
  {/if}
</div>
