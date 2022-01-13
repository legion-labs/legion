<script lang="ts">
  import { PropertyUpdate } from "@/api";

  import { ResourceProperty } from "@/lib/propertyGrid";
  import PropertyInput from "./PropertyInput.svelte";

  type $$Events = {
    input: CustomEvent<PropertyUpdate>;
  };

  export let property: ResourceProperty;

  /** The property path parts */
  export let pathParts: string[];

  export let withBorder: boolean;
</script>

<div class="root" class:with-border={withBorder}>
  {#if property.name}
    <div class="property-name" title={property.name}>
      <div class="truncate">{property.name}</div>
    </div>
  {/if}
  <div class="property-input-container">
    <div class="property-input">
      <PropertyInput
        on:input
        pathParts={[...pathParts, property.name]}
        {property}
      />
    </div>
    <div class="property-actions" />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row py-1 pl-1 space-x-1 justify-between;
  }

  .with-border {
    @apply border-b border-gray-400 border-opacity-30;
  }

  .property-name {
    @apply flex w-1/2 flex-shrink font-semibold text-lg min-w-0;
  }

  .property-input-container {
    @apply flex w-1/2 min-w-[9rem] flex-shrink-0 space-x-1;
  }

  .property-input {
    @apply flex w-full justify-end;
  }

  .property-actions {
    @apply flex flex-row items-center;
  }
</style>
