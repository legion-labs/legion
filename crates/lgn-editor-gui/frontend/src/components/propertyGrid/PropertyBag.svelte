<script lang="ts">
  import { PropertyUpdate } from "@/api";
  import {
    BagResourceProperty,
    buildDefaultPrimitiveProperty,
    propertyIsDynComponent,
    propertyIsGroup,
    propertyIsOption,
    propertyIsVec,
  } from "@/lib/propertyGrid";
  import currentResource from "@/stores/currentResource";
  import { createEventDispatcher } from "svelte";
  import log from "@lgn/web-client/src/lib/log";
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
  const { data: currentResourceData } = currentResource;

  // TODO: Optional property bags are disabled until they're properly supported
  const disabledOptionalProperty = true;

  // Option resource property can be groups
  export let property: BagResourceProperty;

  export let parentProperty: BagResourceProperty | null;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];

  function addVectorSubProperty() {
    const index = property.subProperties.length;
    dispatch("addVectorSubProperty", {
      path: [...pathParts, property.name].join("."),
      index,
      property,
    });
  }

  function removeComponent() {
    if (!parentProperty) {
      log.error("Vector sub property parent not found");
      return;
    }

    if (!$currentResourceData) {
      log.error(
        "A vector sub property was removed while no resources were selected"
      );
      return;
    }

    for (var i = 0; i < parentProperty.subProperties.length; ++i) {
      if (parentProperty.subProperties[i].name == property.name) {
        dispatch("removeVectorSubProperty", {
          path: pathParts.join("."),
          index: i,
        });
        parentProperty.subProperties.splice(i, 1);
        parentProperty.subProperties = parentProperty.subProperties;
        break;
      }
    }
  }
</script>

<div class="root" class:with-indent={level > 1}>
  {#if property.name}
    <div class="property-name" title={property.name}>
      <div class="truncate">
        {property.name}
      </div>
      {#if parentProperty && propertyIsDynComponent(parentProperty)}
        <div
          class="close-button"
          on:click={removeComponent}
          title="Remove Component"
        >
          &#215;
        </div>
      {/if}
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
  {#each property.subProperties as subProperty, index (`${subProperty.name}-${index}`)}
    {#if !subProperty.attributes.hidden}
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
    {/if}
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

  .close-button {
    @apply flex flex-row flex-shrink-0 items-center justify-center h-5 w-6 rounded-sm text-xl bg-gray-700 ml-1 cursor-pointer;
  }
</style>
