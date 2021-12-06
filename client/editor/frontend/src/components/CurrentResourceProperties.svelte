<script lang="ts">
  import {
    propertyIsBoolean,
    propertyIsColor,
    propertyIsNumber,
    propertyIsQuat,
    propertyIsSpeed,
    propertyIsString,
    propertyIsVec3,
    propertyIsVecU8,
    PropertyUpdate,
    updateResourceProperties,
  } from "@/api";
  import currentResource from "@/stores/currentResource";
  import BooleanProperty from "./properties/BooleanProperty.svelte";
  import ColorProperty from "./properties/ColorProperty.svelte";
  import NumberProperty from "./properties/NumberProperty.svelte";
  import QuatProperty from "./properties/QuatProperty.svelte";
  import SpeedProperty from "./properties/SpeedProperty.svelte";
  import StringProperty from "./properties/StringProperty.svelte";
  import Vec3Property from "./properties/Vec3Property.svelte";

  const propertyUpdateDebounceTimeout = 300;

  let updateTimeout: ReturnType<typeof setTimeout> | null = null;

  let propertyUpdates: PropertyUpdate[] = [];

  function onInput(propertyUpdate: PropertyUpdate) {
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
    <div>
      {#each $currentResource.properties as property (property.name)}
        <div class="property">
          <div class="property-name" title={property.name}>
            <div class="truncate">{property.name}</div>
          </div>
          <div class="property-input-container">
            <div class="property-input">
              {#if propertyIsBoolean(property)}
                <BooleanProperty
                  on:input={({ detail }) =>
                    onInput({ name: property.name, value: detail })}
                  bind:value={property.value}
                />
              {:else if propertyIsString(property)}
                <StringProperty
                  on:input={({ detail }) =>
                    onInput({ name: property.name, value: detail })}
                  bind:value={property.value}
                />
              {:else if propertyIsNumber(property)}
                <NumberProperty
                  on:input={({ detail }) =>
                    onInput({ name: property.name, value: detail })}
                  bind:value={property.value}
                />
              {:else if propertyIsColor(property)}
                <ColorProperty
                  on:input={({ detail }) =>
                    onInput({ name: property.name, value: detail })}
                  bind:value={property.value}
                />
              {:else if propertyIsSpeed(property)}
                <SpeedProperty
                  on:input={({ detail }) =>
                    onInput({ name: property.name, value: detail })}
                  bind:value={property.value}
                />
              {:else if propertyIsVec3(property)}
                <Vec3Property
                  on:input={({ detail }) =>
                    onInput({ name: property.name, value: detail })}
                  bind:value={property.value}
                />
              {:else if propertyIsQuat(property)}
                <QuatProperty
                  on:input={({ detail }) =>
                    onInput({ name: property.name, value: detail })}
                  bind:value={property.value}
                />
              {:else if propertyIsVecU8(property)}
                Vec: {property.value}
              {:else}
                Unknown property: {JSON.stringify(property)}
              {/if}
            </div>
            <div
              class="property-actions"
              on:click={() => {
                property.value = property.defaultValue;

                onInput({ name: property.name, value: property.defaultValue });
              }}
            >
              <div
                class="property-action-default"
                title="Reset value to default"
              >
                &#10227;
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply px-2;
  }

  .property {
    @apply flex flex-row border-b py-1 border-gray-400 border-opacity-30 last:border-none space-x-1 justify-between;
  }

  .property-name {
    @apply flex flex-shrink font-bold text-lg min-w-0 w-auto;
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

  .property-action-default {
    @apply cursor-pointer text-lg;
  }
</style>
