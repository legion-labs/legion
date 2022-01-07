<script lang="ts">
  import {
    propertyIsBoolean,
    propertyIsColor,
    propertyIsGroup,
    propertyIsNumber,
    propertyIsQuat,
    propertyIsSpeed,
    propertyIsString,
    propertyIsVec3,
    propertyIsVecU8,
    propertyIsVirtualGroup,
    PropertyUpdate,
    ResourceProperty,
    ResourcePropertyGroup,
  } from "@/api";
  import { createEventDispatcher } from "svelte";
  import BooleanProperty from "./properties/BooleanProperty.svelte";
  import ColorProperty from "./properties/ColorProperty.svelte";
  import NumberProperty from "./properties/NumberProperty.svelte";
  import QuatProperty from "./properties/QuatProperty.svelte";
  import SpeedProperty from "./properties/SpeedProperty.svelte";
  import StringProperty from "./properties/StringProperty.svelte";
  import Vec3Property from "./properties/Vec3Property.svelte";

  const dispatch = createEventDispatcher<{ input: PropertyUpdate }>();

  export let property: ResourceProperty | ResourcePropertyGroup;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];

  /** Displays a nice little border below the resource property! */
  export let withBorder: boolean;

  function onInput({ value }: PropertyUpdate) {
    dispatch("input", {
      name: pathParts.filter(Boolean).join("."),
      value,
    });
  }
</script>

{#if propertyIsGroup(property)}
  <div
    class="root-group"
    class:with-indent={level > 1}
    class:with-border={withBorder}
  >
    {#if property.name}
      <div
        class:property-group-name={property.ptype === "virtual-group"}
        class:property-sub-name={property.ptype === "group"}
        title={property.name}
      >
        <div class="truncate">{property.name}</div>
      </div>
    {/if}
    {#each property.subProperties as subProperty, index (subProperty.name)}
      <svelte:self
        pathParts={propertyIsVirtualGroup(subProperty)
          ? pathParts
          : [...pathParts, subProperty.name]}
        property={subProperty}
        on:input
        level={level + 1}
        withBorder={property.subProperties[index + 1] &&
          !propertyIsGroup(property.subProperties[index + 1])}
      />
    {/each}
  </div>
{:else}
  <div class="root-property" class:with-border={withBorder}>
    {#if property.name}
      <div class="property-name" title={property.name}>
        <div class="truncate">{property.name}</div>
      </div>
    {/if}
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
          Vec: {property.name} - {property.value}
        {:else}
          <div class="unknown-property">
            Unknown property: {JSON.stringify(property, null, 2)}
          </div>
        {/if}
      </div>
      <div class="property-actions" />
    </div>
  </div>
{/if}

<style lang="postcss">
  .root-group {
    @apply flex flex-col pt-1 last:pb-1 justify-between;
  }

  .root-group.with-indent {
    @apply pl-1;
  }

  .root-property {
    @apply flex flex-row py-1 pl-1 space-x-1 justify-between;
  }

  .with-border {
    @apply border-b border-gray-400 border-opacity-30 last:border-none;
  }

  .property-group-name,
  .property-sub-name {
    @apply flex flex-row items-center h-7 px-1 mb-0.5 font-bold bg-gray-800 rounded-sm;
  }

  .property-name {
    @apply flex w-1/2 flex-shrink font-semibold text-lg min-w-0;
  }

  .property-input-container {
    @apply flex w-1/2 min-w-[9rem] flex-shrink-0 space-x-1;
  }

  .property-input {
    @apply flex w-full justify-end;
  }

  .unknown-property {
    @apply break-all;
  }

  .property-actions {
    @apply flex flex-row items-center;
  }
</style>
