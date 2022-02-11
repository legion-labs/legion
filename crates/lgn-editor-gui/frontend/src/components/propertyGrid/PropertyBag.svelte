<script lang="ts">
  import { PropertyUpdate } from "@/api";
  import {
    BagResourceProperty,
    buildDefaultPrimitiveProperty,
    propertyIsGroup,
    propertyIsOption,
    propertyIsVec,
  } from "@/lib/propertyGrid";
  import { createEventDispatcher } from "svelte";
  import Checkbox from "../inputs/Checkbox.svelte";
  import PropertyContainer from "./PropertyContainer.svelte";
  import {
    AddVectorSubPropertyEvent,
    RemoveVectorSubPropertyEvent,
  } from "./types";

  const dispatch = createEventDispatcher<{
    input: PropertyUpdate;
    addVectorSubProperty: AddVectorSubPropertyEvent;
    removeVectorSubProperty: RemoveVectorSubPropertyEvent;
  }>();

  // TODO: Optional property bags are disabled until they're properly supported
  const disabledOptionalProperty = true;

  // Option resource property can be groups
  export let property: BagResourceProperty;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];

  function addVectorSubProperty() {
    const index = property.subProperties.length;

    const addedProperty = buildDefaultPrimitiveProperty(`[${index}]`, "u8");

    property.subProperties = [...property.subProperties, addedProperty];

    dispatch("addVectorSubProperty", {
      path: [...pathParts, property.name].join("."),
      index,
      value: addedProperty.value,
    });
  }
</script>

<div class="root" class:with-indent={level > 1}>
  {#if property.name}
    <div class="property-name" title={property.name}>
      <div class="truncate">{property.name}</div>
      {#if !disabledOptionalProperty && propertyIsOption(property)}
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
  {#each property.subProperties as subProperty, index (subProperty.name)}
    <PropertyContainer
      on:input
      on:addVectorSubProperty
      on:removeVectorSubProperty
      pathParts={propertyIsGroup(property) || !property.name
        ? pathParts
        : [...pathParts, property.name]}
      property={subProperty}
      bind:parentProperty={property}
      level={level + 1}
      {index}
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
