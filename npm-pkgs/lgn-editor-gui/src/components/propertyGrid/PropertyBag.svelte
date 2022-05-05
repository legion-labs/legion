<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import log from "@lgn/web-client/src/lib/log";

  import type { PropertyUpdate } from "@/api";
  import {
    propertyIsDynComponent,
    propertyIsGroup,
    propertyIsOption,
    propertyIsVec,
  } from "@/lib/propertyGrid";
  import type { BagResourceProperty } from "@/lib/propertyGrid";
  import { currentResource } from "@/orchestrators/currentResource";
  import modal from "@/stores/modal";

  import Checkbox from "../inputs/Checkbox.svelte";
  import PropertyContainer from "./PropertyContainer.svelte";
  import type {
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

  export let parentProperty: BagResourceProperty | null;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];

  let removePromptId: symbol | null = null;

  function addVectorSubProperty() {
    const index = property.subProperties.length;

    dispatch("addVectorSubProperty", {
      path: [...pathParts, property.name].join("."),
      index,
      property,
    });
  }

  function requestRemoveComponent() {
    removePromptId = Symbol.for("request-component-remove");

    modal.prompt(removePromptId);
  }

  function removeComponent({
    detail,
  }: CustomEvent<{ answer: boolean; id: symbol }>) {
    if (!removePromptId) {
      return;
    }

    const id = removePromptId;

    removePromptId = null;

    if (id !== detail.id || !detail.answer) {
      return;
    }

    if (!parentProperty) {
      log.error("Vector sub property parent not found");

      return;
    }

    if (!$currentResource) {
      log.error(
        "A vector sub property was removed while no resources were selected"
      );

      return;
    }

    const subPropertyIndex = parentProperty.subProperties.findIndex(
      (subProperty) => subProperty.name === property.name
    );

    if (subPropertyIndex < 0) {
      log.error(
        log.json`Sub property with name ${property.name} not found in ${property}`
      );

      return;
    }

    dispatch("removeVectorSubProperty", {
      path: pathParts.join("."),
      index: subPropertyIndex,
    });

    parentProperty.subProperties.splice(subPropertyIndex, 1);
    parentProperty.subProperties = parentProperty.subProperties;
  }

  function beautifyComponentName(name: string) {
    return name.replace("[", "").replace("]", "");
  }
</script>

<svelte:window on:prompt-answer={removeComponent} />

<div class="flex flex-col justify-between root">
  {#if property.name}
    <div
      style="padding-left:{level / 4}rem"
      class="flex flex-row items-center justify-between pl-0 h-8 font-semibold bg-surface-900"
      title={property.name}
    >
      <div class="truncate my-auto">
        {beautifyComponentName(property.name)}
      </div>
      {#if parentProperty && propertyIsDynComponent(parentProperty)}
        <div
          class="delete-button"
          on:click={requestRemoveComponent}
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
  .optional {
    @apply flex items-center justify-center h-7 w-7 border-l-2 border-gray-700 cursor-pointer;
  }

  .add-vector {
    @apply flex items-center justify-center h-7 w-7 border-l-2 border-gray-700 bg-green-800 bg-opacity-70 rounded-r-sm cursor-pointer;
  }

  .delete-button {
    @apply flex items-center justify-center h-7 w-7 border-l-2 border-gray-700 bg-red-800 bg-opacity-70 rounded-r-sm cursor-pointer;
  }
</style>
