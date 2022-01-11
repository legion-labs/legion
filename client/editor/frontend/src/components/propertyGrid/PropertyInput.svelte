<script lang="ts">
  import { PropertyUpdate } from "@/api";
  import {
    propertyIsBoolean,
    propertyIsColor,
    propertyIsNumber,
    propertyIsOption,
    propertyIsQuat,
    propertyIsSpeed,
    propertyIsString,
    propertyIsVec,
    propertyIsVec3,
    ResourceProperty,
  } from "@/api/propertyGrid";
  import { createEventDispatcher } from "svelte";
  import BooleanProperty from "./properties/BooleanProperty.svelte";
  import ColorProperty from "./properties/ColorProperty.svelte";
  import NumberProperty from "./properties/NumberProperty.svelte";
  import QuatProperty from "./properties/QuatProperty.svelte";
  import SpeedProperty from "./properties/SpeedProperty.svelte";
  import StringProperty from "./properties/StringProperty.svelte";
  import Vec3Property from "./properties/Vec3Property.svelte";

  const dispatch = createEventDispatcher<{ input: PropertyUpdate }>();

  export let property: ResourceProperty;

  /** The property path parts */
  export let pathParts: string[];

  function onInput({ value }: Pick<PropertyUpdate, "value">) {
    dispatch("input", {
      name: pathParts.filter(Boolean).join("."),
      value,
    });
  }
</script>

{#if propertyIsBoolean(property)}
  <BooleanProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
  />
{:else if propertyIsString(property)}
  <StringProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
  />
{:else if propertyIsNumber(property)}
  <NumberProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
  />
{:else if propertyIsColor(property)}
  <ColorProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
  />
{:else if propertyIsSpeed(property)}
  <SpeedProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
  />
{:else if propertyIsVec3(property)}
  <Vec3Property
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
  />
{:else if propertyIsQuat(property)}
  <QuatProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
  />
{:else if propertyIsVec(property)}
  {property.ptype} not implemented
{:else if propertyIsOption(property)}
  {#if property.subProperties[0]}
    <svelte:self on:input {pathParts} property={property.subProperties[0]} />
  {:else}
    Property not set
  {/if}
{:else}
  <div class="unknown-property">
    Unknown property: {JSON.stringify(property, null, 2)}
  </div>
{/if}

<style lang="postcss">
  .unknown-property {
    @apply break-all;
  }
</style>
