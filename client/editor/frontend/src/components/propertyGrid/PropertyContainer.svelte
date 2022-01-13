<script lang="ts">
  import { propertyIsBag, ResourceProperty } from "@/lib/propertyGrid";
  import PropertyBag from "./PropertyBag.svelte";
  import PropertyUnit from "./PropertyUnit.svelte";

  export let property: ResourceProperty;

  export let nextProperty: ResourceProperty | undefined;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];

  let nextPropertyIsBag =
    (nextProperty && propertyIsBag(nextProperty)) || false;
</script>

<div class="root">
  {#if propertyIsBag(property)}
    <PropertyBag
      on:input
      {property}
      {level}
      {pathParts}
      withBorder={(nextProperty && !nextPropertyIsBag) || false}
    />
  {:else}
    <PropertyUnit
      on:input
      {property}
      {pathParts}
      withBorder={(nextProperty && !nextPropertyIsBag) || false}
    />
  {/if}
</div>
