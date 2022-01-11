<script lang="ts">
  import { PropertyUpdate } from "@/api";

  import {
    ComponentResourceProperty,
    GroupResourceProperty,
    OptionResourceProperty,
    propertyIsGroup,
  } from "@/api/propertyGrid";
  import PropertyContainer from "./PropertyContainer.svelte";

  type $$Events = {
    input: CustomEvent<PropertyUpdate>;
  };

  // Option resource property can be groups
  export let property:
    | GroupResourceProperty
    | ComponentResourceProperty
    | OptionResourceProperty;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];

  /** Displays a nice little border below the resource property (or not)! */
  export let withBorder: boolean;
</script>

<div class="root" class:with-indent={level > 1} class:with-border={withBorder}>
  {#if property.name}
    <div class="property-name" title={property.name}>
      <div class="truncate">{property.name}</div>
    </div>
  {/if}
  {#each property.subProperties as subProperty, index (subProperty.name)}
    <PropertyContainer
      pathParts={propertyIsGroup(property)
        ? pathParts
        : [...pathParts, property.name]}
      property={subProperty}
      on:input
      level={level + 1}
      withBorder={property.subProperties[index + 1] &&
        !propertyIsGroup(property.subProperties[index + 1])}
    />
  {/each}
</div>

<style lang="postcss">
  .root {
    @apply flex flex-col pt-1 justify-between;
  }

  .root.with-indent {
    @apply pl-1;
  }

  .with-border {
    @apply border-b border-gray-400 border-opacity-30 last:border-none;
  }

  .property-name {
    @apply flex flex-row items-center h-7 px-1 mb-0.5 font-bold bg-gray-800 rounded-sm;
  }
</style>
