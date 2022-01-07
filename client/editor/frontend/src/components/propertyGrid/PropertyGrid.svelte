<script lang="ts">
  import {
    propertyIsGroup,
    propertyIsVirtualGroup,
    PropertyUpdate,
    updateResourceProperties,
  } from "@/api";
  import currentResource from "@/stores/currentResource";
  import Property from "./Property.svelte";

  const propertyUpdateDebounceTimeout = 300;

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

      if (!$currentResource) {
        return;
      }

      updateResourceProperties(
        $currentResource.id,
        $currentResource.version,
        propertyUpdates
      );

      propertyUpdates = [];
    }, propertyUpdateDebounceTimeout);
  }
</script>

<div class="root">
  {#if !$currentResource}
    <div class="italic">No resource selected</div>
  {:else if !$currentResource.properties.length}
    <div class="italic">Resource has no properties</div>
  {:else}
    {#each $currentResource.properties as property, index (property.name)}
      <Property
        pathParts={propertyIsVirtualGroup(property) ? [] : [property.name]}
        {property}
        withBorder={$currentResource.properties[index + 1] &&
          !propertyIsGroup($currentResource.properties[index + 1])}
        on:input={onInput}
      />
    {/each}
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply h-full px-1 overflow-y-auto;
  }
</style>
