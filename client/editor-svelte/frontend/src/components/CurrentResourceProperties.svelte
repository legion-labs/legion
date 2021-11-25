<script lang="ts">
  import currentResource from "@/stores/currentResource";
  import BooleanProperty from "./properties/BooleanProperty.svelte";
  import ColorProperty from "./properties/ColorProperty.svelte";
  import NumberProperty from "./properties/NumberProperty.svelte";
  import StringProperty from "./properties/StringProperty.svelte";

  const ptypeIsBoolean = (ptype: string) =>
    ["bool"].includes(ptype.toLowerCase());

  const ptypeIsSpeed = (ptype: string) =>
    ["speep"].includes(ptype.toLowerCase());

  const ptypeIsColor = (ptype: string) =>
    ["color"].includes(ptype.toLowerCase());

  const ptypeIsString = (ptype: string) =>
    ["string"].includes(ptype.toLowerCase());

  const ptypeIsNumber = (ptype: string) =>
    ["i32", "u32", "f32", "f64"].includes(ptype.toLowerCase());

  const ptypeIsVector = (ptype: string) =>
    ["vec3", "quat", "vec < u8 >"].includes(ptype.toLowerCase());
</script>

<div class="root">
  {#if !$currentResource}
    <div class="italic">No resource selected</div>
  {:else if !$currentResource.properties.length}
    <div class="italic">Resource has no properties</div>
  {:else}
    <div class="properties">
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
              Speed
            {:else if ptypeIsVector(property.ptype)}
              Vector
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

  .properties {
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
