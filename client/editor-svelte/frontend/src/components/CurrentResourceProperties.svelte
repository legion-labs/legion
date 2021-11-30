<script lang="ts">
  import currentResource from "@/stores/currentResource";
  import BooleanProperty from "./properties/BooleanProperty.svelte";
  import ColorProperty from "./properties/ColorProperty.svelte";
  import NumberProperty from "./properties/NumberProperty.svelte";
  import QuatProperty from "./properties/QuatProperty.svelte";
  import SpeedProperty from "./properties/SpeedProperty.svelte";
  import StringProperty from "./properties/StringProperty.svelte";
  import Vector3Property from "./properties/Vector3Property.svelte";

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
</script>

<div class="root">
  {#if !$currentResource}
    <div class="italic">No resource selected</div>
  {:else if !$currentResource.properties.length}
    <div class="italic">Resource has no properties</div>
  {:else}
    <div>
      <!-- TODO: Make sure the name is unique -->
      {#each $currentResource.properties as property (property.name)}
        <div class="property">
          <div class="property-name">
            {property.name}
          </div>
          <div class="property-input-container">
            <div class="property-input">
              {#if ptypeIsBoolean(property.ptype)}
                <BooleanProperty bind:value={property.value} />
              {:else if ptypeIsString(property.ptype)}
                <StringProperty bind:value={property.value} />
              {:else if ptypeIsNumber(property.ptype)}
                <NumberProperty bind:value={property.value} />
              {:else if ptypeIsColor(property.ptype)}
                <ColorProperty bind:value={property.value} />
              {:else if ptypeIsSpeed(property.ptype)}
                <SpeedProperty bind:value={property.value} />
              {:else if ptypeIsVector3(property.ptype)}
                <Vector3Property bind:value={property.value} />
              {:else if ptypeIsQuat(property.ptype)}
                <QuatProperty bind:value={property.value} />
              {:else if ptypeIsVecU8(property.ptype)}
                Vec: {property.value}
              {:else}
                Unknown property type: {property.ptype}
              {/if}
            </div>
            <div
              class="property-actions"
              on:click={() => (property.value = property.defaultValue)}
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
    @apply font-bold text-lg;
  }

  .property-actions {
    @apply flex flex-row space-x-1 items-center;
  }

  .property-action-default {
    @apply cursor-pointer text-lg;
  }

  .property-input-container {
    @apply flex w-1/2 min-w-[160px] flex-shrink-0 space-x-1;
  }

  .property-input {
    @apply flex w-full;
  }
</style>
