<script lang="ts">
  import { PropertyUpdate } from "@/api";
  import {
    buildDefaultPrimitiveProperty,
    extractOptionPType,
    propertyIsBoolean,
    propertyIsColor,
    propertyIsNumber,
    propertyIsOption,
    propertyIsQuat,
    propertyIsSpeed,
    propertyIsString,
    propertyIsVec,
    propertyIsVec3,
    ptypeBelongsToPrimitive,
    ResourceProperty,
  } from "@/lib/propertyGrid";
  import { createEventDispatcher } from "svelte";
  import Checkbox from "../inputs/Checkbox.svelte";
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

  // Option related code
  // TODO: Try to extract the option input in its own component.
  // It seems to fail because of the mutual recursion though.
  /**
   * Related to the option property, set to `null` if the property is not an option.
   * Set to `true` if the option property contains a sub properties, and is therefore `Some`.
   */
  let isSome: boolean | null = null;

  // We need to disambiguate `false`/`null` here
  // `false` means the property is of type `Option` with value `None`
  // `null` means the property is not an `Option`
  export let disabled = isSome === false;

  function setOptionProperty({ detail: isSome }: CustomEvent<boolean>) {
    // TODO: Send an input event that be can sent to the server

    // Not supposed to happen, we can consider casting
    // the property as an option resource property at that point
    if (!propertyIsOption(property)) {
      return;
    }

    disabled = !isSome;

    if (isSome) {
      const innerPType = extractOptionPType(property);

      // TODO: Handle non primitives
      if (innerPType && ptypeBelongsToPrimitive(innerPType)) {
        property.subProperties[0] = buildDefaultPrimitiveProperty(
          property.name,
          innerPType
        );
      }
    } else {
      delete property.subProperties[0];

      onInput({ value: null });
    }
  }
</script>

{#if propertyIsBoolean(property)}
  <BooleanProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
    {disabled}
  />
{:else if propertyIsString(property)}
  <StringProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
    {disabled}
  />
{:else if propertyIsNumber(property)}
  <NumberProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
    {disabled}
  />
{:else if propertyIsColor(property)}
  <ColorProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
    {disabled}
  />
{:else if propertyIsSpeed(property)}
  <SpeedProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
    {disabled}
  />
{:else if propertyIsVec3(property)}
  <Vec3Property
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
    {disabled}
  />
{:else if propertyIsQuat(property)}
  <QuatProperty
    on:input={({ detail }) => onInput({ value: detail })}
    bind:value={property.value}
    {disabled}
  />
{:else if propertyIsVec(property)}
  {property.ptype} not implemented
{:else if propertyIsOption(property)}
  {#if property.subProperties[0]}
    <div class="option-property">
      <svelte:self
        on:input
        {pathParts}
        property={property.subProperties[0]}
        {disabled}
      />
      <div class="option-property-checkbox">
        <Checkbox on:change={setOptionProperty} value={true} />
      </div>
    </div>
  {:else}
    <div class="option-property">
      <div
        class="cursor-help"
        title="This property's value is optional and no value has been set yet"
      >
        <i>Value not set</i>
        <div class="cursor-help-icon">?</div>
      </div>
      <div class="option-property-checkbox">
        <Checkbox on:change={setOptionProperty} value={false} />
      </div>
    </div>
  {/if}
{:else}
  <div class="unknown-property">
    Unknown property: {property.ptype}
  </div>
{/if}

<style lang="postcss">
  .unknown-property {
    @apply break-all;
  }

  .option-property {
    @apply flex flex-row;
  }

  .option-property-checkbox {
    @apply flex items-center flex-shrink-0 h-full pl-1;
  }

  .cursor-help {
    @apply flex flex-row h-8 space-x-1 items-center;
  }

  .cursor-help-icon {
    @apply flex flex-row self-start justify-center items-center text-xs h-3 w-3 bg-gray-500 rounded-full;
  }
</style>
