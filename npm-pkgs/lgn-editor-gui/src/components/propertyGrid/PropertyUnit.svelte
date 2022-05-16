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

  export let level: number;

  function beautifyPropertyName(name: string) {
    const split = name.split("_");

    for (let i = 0; i < split.length; i++) {
      split[i] = split[i][0].toUpperCase() + split[i].slice(1, split[i].length);
    }

    return split.join(" ");
  }
</script>

<div
  class="flex flex-row h-8 space-x-1 justify-between"
  style="padding-left:{level / 4}rem"
  class:bg-surface-700={index % 2 === 0}
  class:bg-surface-800={index % 2 !== 0}
>
  {#if property.name}
    <div class="flex w-full flex-grow m-auto min-w-0" title={property.name}>
      <div class="truncate">{beautifyPropertyName(property.name)}</div>
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
  .property-input-container {
    @apply flex w-[10rem] flex-shrink-0 flex-grow-0;
  }

  .property-input {
    @apply flex w-full justify-end;
  }
</style>
