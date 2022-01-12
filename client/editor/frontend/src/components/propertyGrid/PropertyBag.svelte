<script lang="ts">
  import { PropertyUpdate } from "@/api";
  import {
    BagResourceProperty,
    propertyIsGroup,
    propertyIsOption,
  } from "@/lib/propertyGrid";
  import Checkbox from "../inputs/Checkbox.svelte";
  import PropertyContainer from "./PropertyContainer.svelte";

  type $$Events = {
    input: CustomEvent<PropertyUpdate>;
  };

  // Option resource property can be groups
  export let property: BagResourceProperty;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];

  export let withBorder: boolean;
</script>

<div class="root" class:with-indent={level > 1} class:with-border={withBorder}>
  {#if property.name}
    <div class="property-name" title={property.name}>
      <div class="truncate">{property.name}</div>
      <!-- TODO: Optional property bags are disabled until they're properly supported -->
      {#if false && propertyIsOption(property)}
        <div class="optional">
          <Checkbox value={true} />
        </div>
      {/if}
    </div>
  {/if}
  {#each property.subProperties as subProperty, index (subProperty.name)}
    <PropertyContainer
      pathParts={propertyIsGroup(property)
        ? pathParts
        : [...pathParts, property.name]}
      property={subProperty}
      nextProperty={property.subProperties[index + 1]}
      on:input
      level={level + 1}
    />
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col justify-between;
  }

  .root.with-indent {
    @apply pl-1;
  }

  .with-border {
    @apply border-b border-gray-400 border-opacity-30;
  }

  .property-name {
    @apply flex flex-row items-center justify-between h-7 pl-1 my-1 font-bold bg-gray-800 rounded-sm;
  }

  .optional {
    @apply flex items-center justify-center h-7 w-7 border-l-2 border-gray-700;
  }
</style>
