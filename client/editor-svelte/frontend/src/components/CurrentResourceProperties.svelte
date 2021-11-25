<script lang="ts">
  import currentResource from "@/stores/currentResource";
  import BooleanProperty from "./properties/BooleanProperty.svelte";
  import ColorProperty from "./properties/ColorProperty.svelte";
  import NumberProperty from "./properties/NumberProperty.svelte";
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
            {:else}
              Unknown property type: {property.ptype}
            {/if}
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
    @apply flex flex-col border-b py-2 border-gray-400 last:border-none space-y-0.5;
  }

  .property-name {
    @apply font-bold text-sm;
  }

  .property-input {
    @apply flex text-sm w-full;
  }
</style>
