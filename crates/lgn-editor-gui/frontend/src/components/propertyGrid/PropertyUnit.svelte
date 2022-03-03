<script lang="ts">
  import type { PropertyUpdate } from "@/api";
  import type {
    BagResourceProperty,
    ResourceProperty,
  } from "@/lib/propertyGrid";
  import PropertyInput from "./PropertyInput.svelte";
  import type { RemoveVectorSubPropertyEvent } from "./types";

  type $$Events = {
    input: CustomEvent<PropertyUpdate>;
    removeVectorSubProperty: CustomEvent<RemoveVectorSubPropertyEvent>;
  };

  export let property: ResourceProperty;

  export let parentProperty: BagResourceProperty | null;

  /** The property path parts */
  export let pathParts: string[];

  /** The property index (only used in vectors) */
  export let index: number;
</script>

<div class="root">
  {#if property.name}
    <div class="property-name" title={property.name}>
      <div class="truncate">{property.name}</div>
    </div>
  {/if}
  <div class="property-input-container">
    <div class="property-input">
      <PropertyInput
        on:input
        on:removeVectorSubProperty
        pathParts={property.name ? [...pathParts, property.name] : pathParts}
        {property}
        {index}
        bind:parentProperty
      />
    </div>
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row py-0.5 pl-1 space-x-1 justify-between;
  }

  .property-name {
    @apply flex w-full flex-grow text-lg min-w-0 border-b-[0.5px] border-dashed border-gray-400;
  }

  .property-input-container {
    @apply flex w-[10rem] flex-shrink-0 flex-grow-0;
  }

  .property-input {
    @apply flex w-full justify-end;
  }
</style>
