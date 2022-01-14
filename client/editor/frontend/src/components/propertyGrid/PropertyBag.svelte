<script lang="ts">
  import { PropertyUpdate } from "@/api";
  import {
    BagResourceProperty,
    buildDefaultPrimitiveProperty,
    propertyIsGroup,
    propertyIsOption,
    propertyIsVec,
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

  /** Adds a new property to a vector, only useful for vectors */
  function addVectorSubProperty() {
    // TODO: Dispatch event and use rpc add item

    property.subProperties = [
      ...property.subProperties,
      buildDefaultPrimitiveProperty(`[${property.subProperties.length}]`, "u8"),
    ];
  }

  /** Removes a new property from a vector, only useful for vectors */
  function removeVectorSubProperty({ detail: name }: CustomEvent<string>) {
    // TODO: Dispatch event and use rpc remove item

    property.subProperties = property.subProperties
      .filter((property) => property.name !== name)
      .map((property, index) => ({ ...property, name: `[${index}]` }));
  }
</script>

<div class="root" class:with-indent={level > 1}>
  {#if property.name}
    <div class="property-name" title={property.name}>
      <div class="truncate">{property.name}</div>
      <!-- TODO: Optional property bags are disabled until they're properly supported -->
      {#if false && propertyIsOption(property)}
        <div class="optional">
          <Checkbox value={true} />
        </div>
      {/if}
      {#if propertyIsVec(property)}
        <div
          class="add-vector"
          on:click={addVectorSubProperty}
          title="Add property to vector"
        >
          +
        </div>
      {/if}
    </div>
  {/if}
  {#each property.subProperties as subProperty (subProperty.name)}
    <PropertyContainer
      on:input
      on:removeVectorProperty={removeVectorSubProperty}
      pathParts={propertyIsGroup(property)
        ? pathParts
        : [...pathParts, property.name]}
      property={subProperty}
      parentProperty={property}
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

  .property-name {
    @apply flex flex-row items-center justify-between h-7 pl-1 my-0.5 font-semibold bg-gray-800 rounded-sm;
  }

  .optional {
    @apply flex items-center justify-center h-7 w-7 border-l-2 border-gray-700 cursor-pointer;
  }

  .add-vector {
    @apply flex items-center justify-center h-7 w-7 border-l-2 border-gray-700 cursor-pointer;
  }
</style>
