<script lang="ts">
  import { PropertyUpdate, updateResourceProperties } from "@/api";
  import { propertyIsGroup } from "@/lib/propertyGrid";
  import currentResource from "@/stores/currentResource";
  import PropertyContainer from "./PropertyContainer.svelte";

  const { data: currentResourceData, error: currentResourceError } =
    currentResource;

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
</script>

<div class="root">
  {#if $currentResourceError}
    <div class="italic">An error occured</div>
  {:else if !$currentResourceData}
    <div class="italic">No resource selected</div>
  {:else if !$currentResourceData.properties.length}
    <div class="italic">Resource has no properties</div>
  {:else}
    {#each $currentResourceData.properties as property (property.name)}
      <PropertyContainer
        pathParts={propertyIsGroup(property) ? [] : [property.name]}
        {property}
        parentProperty={null}
        on:input={onInput}
      />
    {/each}
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply h-full px-1 py-1 overflow-y-auto;
  }
</style>
