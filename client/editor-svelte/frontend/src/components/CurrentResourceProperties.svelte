<script lang="ts">
  import { PropertyUpdate, updateResourceProperties } from "@/api";
  import currentResource from "@/stores/currentResource";
  import BooleanProperty from "./properties/BooleanProperty.svelte";
  import ColorProperty from "./properties/ColorProperty.svelte";
  import NumberProperty from "./properties/NumberProperty.svelte";
  import QuatProperty from "./properties/QuatProperty.svelte";
  import SpeedProperty from "./properties/SpeedProperty.svelte";
  import StringProperty from "./properties/StringProperty.svelte";
  import Vector3Property from "./properties/Vector3Property.svelte";

  const propertyUpdateDebounceTimeout = 300;

  const ptypeIsBoolean = (ptype: string) =>
    ["bool"].includes(ptype.toLowerCase());

  const ptypeIsSpeed = (ptype: string) =>
    ["speed"].includes(ptype.toLowerCase());

  const ptypeIsColor = (ptype: string) =>
    ["color"].includes(ptype.toLowerCase());

  const ptypeIsString = (ptype: string) =>
    ["string"].includes(ptype.toLowerCase());

  const ptypeIsNumber = (ptype: string) =>
    ["i32", "u32", "f32", "f64"].includes(ptype.toLowerCase());

  const ptypeIsVector3 = (ptype: string) =>
    ["vec3"].includes(ptype.toLowerCase());

  const ptypeIsQuat = (ptype: string) => ["quat"].includes(ptype.toLowerCase());

  const ptypeIsVecU8 = (ptype: string) =>
    ["vec < u8 >"].includes(ptype.toLowerCase());

  let updateTimeout: ReturnType<typeof setTimeout> | null = null;

  // TODO: Improve type safety
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let propertyUpdates: PropertyUpdate[] = [];

  // TODO: Improve type safety
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function onInput(name: string, ptype: string, value: any) {
    if (updateTimeout) {
      clearTimeout(updateTimeout);
    }

    const newPropertyUpdate = {
      name,
      value: ptype === "color" ? parseInt(value, 16) : value,
    };

    // We save all the property updates performed in a batch.
    // In order to do so we need to know if the property that
    // just got modified was pristine or not, so we look for it
    // in the property updates array.
    const propertyUpdateIndex = propertyUpdates.findIndex(
      (propertyUpdate) => propertyUpdate.name === name
    );

    if (propertyUpdateIndex < 0) {
      // If the property has not been modified since the debounce timeout
      // started then we can push the new update to the known property updates
      propertyUpdates = [...propertyUpdates, newPropertyUpdate];
    } else {
      // Otherwise, we simply need to replace the already modified property value
      // by the new one
      propertyUpdates[propertyUpdateIndex].value = newPropertyUpdate.value;
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
              {#if ptypeIsBoolean(property.ptype)}
                <BooleanProperty
                  on:input={({ detail }) =>
                    onInput(property.name, property.ptype, detail)}
                  bind:value={property.value}
                />
              {:else if ptypeIsString(property.ptype)}
                <StringProperty
                  on:input={({ detail }) =>
                    onInput(property.name, property.ptype, detail)}
                  bind:value={property.value}
                />
              {:else if ptypeIsNumber(property.ptype)}
                <NumberProperty
                  on:input={({ detail }) =>
                    onInput(property.name, property.ptype, detail)}
                  bind:value={property.value}
                />
              {:else if ptypeIsColor(property.ptype)}
                <ColorProperty
                  on:input={({ detail }) =>
                    onInput(property.name, property.ptype, detail)}
                  bind:value={property.value}
                />
              {:else if ptypeIsSpeed(property.ptype)}
                <SpeedProperty
                  on:input={({ detail }) =>
                    onInput(property.name, property.ptype, detail)}
                  bind:value={property.value}
                />
              {:else if ptypeIsVector3(property.ptype)}
                <Vector3Property
                  on:input={({ detail }) =>
                    onInput(property.name, property.ptype, detail)}
                  bind:value={property.value}
                />
              {:else if ptypeIsQuat(property.ptype)}
                <QuatProperty
                  on:input={({ detail }) =>
                    onInput(property.name, property.ptype, detail)}
                  bind:value={property.value}
                />
              {:else if ptypeIsVecU8(property.ptype)}
                Vec: {property.value}
              {:else}
                Unknown property type: {property.ptype}
              {/if}
            </div>
            <div
              class="property-actions"
              on:click={() => {
                property.value = property.defaultValue;

                onInput(property.name, property.ptype, property.defaultValue);
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
