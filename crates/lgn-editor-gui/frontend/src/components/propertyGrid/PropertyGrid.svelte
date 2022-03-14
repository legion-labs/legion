<script lang="ts">
  import {
    updateResourceProperties,
    removeVectorSubProperty as removeVectorSubPropertyApi,
    addPropertyInPropertyVector as addPropertyInPropertyVectorApi,
  } from "@/api";
  import type { PropertyUpdate } from "@/api";
  import { propertyIsDynComponent, propertyIsGroup } from "@/lib/propertyGrid";
  import currentResource from "@/orchestrators/currentResource";
  import log from "@lgn/web-client/src/lib/log";
  import PropertyContainer from "./PropertyContainer.svelte";
  import CreateComponentModal from "@/components/resources/CreateComponentModal.svelte";
  import modal from "@/stores/modal";
  import type {
    AddVectorSubPropertyEvent,
    RemoveVectorSubPropertyEvent,
  } from "./types";

  const { data: currentResourceData, error: currentResourceError } =
    currentResource;

  const createComponentModalId = Symbol();

  const propertyUpdateDebounceTimeout = 100;

  let updateTimeout: ReturnType<typeof setTimeout> | null = null;

  let propertyUpdates: PropertyUpdate[] = [];

  function onInput({ detail: propertyUpdate }: CustomEvent<PropertyUpdate>) {
    if (updateTimeout) {
      clearTimeout(updateTimeout);
    }

    // We save all the property updates performed in a batch.
    // In order to do so we need to know if the property that
    // just got modified was pristine or not, so we look for it
    // in the property updates array.
    const propertyUpdateIndex = propertyUpdates.findIndex(
      ({ name }) => name === propertyUpdate.name
    );

    if (propertyUpdateIndex < 0) {
      // If the property has not been modified since the debounce timeout
      // started then we can push the new update to the known property updates
      propertyUpdates = [...propertyUpdates, propertyUpdate];
    } else {
      // Otherwise, we simply need to replace the already modified property value
      // by the new one
      propertyUpdates[propertyUpdateIndex].value = propertyUpdate.value;
    }

    updateTimeout = setTimeout(() => {
      updateTimeout = null;

      if (!$currentResourceData) {
        return;
      }

      updateResourceProperties(
        $currentResourceData.id,
        $currentResourceData.version,
        propertyUpdates
      );

      propertyUpdates = [];
    }, propertyUpdateDebounceTimeout);
  }

  /** Adds a new property to a vector, only useful for vectors */
  function addVectorSubProperty({
    detail: { path, property, index },
  }: CustomEvent<AddVectorSubPropertyEvent>) {
    if (!$currentResourceData) {
      log.error("No resources selected");

      return;
    }

    if (propertyIsDynComponent(property)) {
      modal.open(createComponentModalId, CreateComponentModal, {
        payload: {
          resourceId: $currentResourceData.id,
          path: path,
          index: index,
        },
      });
    } else {
      addPropertyInPropertyVectorApi($currentResourceData.id, {
        path,
        index,
        jsonValue: undefined,
      });
    }
  }

  /** Removes a new property from a vector, only useful for vectors */
  function removeVectorSubProperty({
    detail: { path, index },
  }: CustomEvent<RemoveVectorSubPropertyEvent>) {
    if (!$currentResourceData) {
      log.error("No resources selected");

      return;
    }

    // TODO: Batch remove?
    removeVectorSubPropertyApi($currentResourceData.id, {
      path,
      indices: [index],
    });
  }
</script>

<div class="root">
  {#if $currentResourceError}
    <div class="italic">An error occured</div>
  {:else if !$currentResourceData}
    <div class="italic">No resource selected</div>
  {:else if !$currentResourceData.properties.length}
    <div class="italic">Resource has no properties</div>
  {:else}
    {#each $currentResourceData.properties as property, index (property.name)}
      {#if !property.attributes.hidden}
        <PropertyContainer
          on:input={onInput}
          on:addVectorSubProperty={addVectorSubProperty}
          on:removeVectorSubProperty={removeVectorSubProperty}
          pathParts={propertyIsGroup(property) || !property.name
            ? []
            : [property.name]}
          {property}
          {index}
          parentProperty={null}
        />
      {/if}
    {/each}
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply h-full w-full px-1 py-1 overflow-y-auto overflow-x-hidden;
  }
</style>
