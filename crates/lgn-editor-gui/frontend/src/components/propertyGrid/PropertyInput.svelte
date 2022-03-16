<script lang="ts">
  import type { PropertyUpdate } from "@/api";
  import {
    propertyIsBoolean,
    propertyIsColor,
    propertyIsNumber,
    propertyIsOption,
    propertyIsQuat,
    propertyIsSpeed,
    propertyIsScript,
    propertyIsString,
    propertyIsResourcePathId,
    propertyIsEnum,
    propertyIsVec,
    propertyIsVec3,
  } from "@/lib/propertyGrid";
  import type {
    BagResourceProperty,
    ResourceProperty,
  } from "@/lib/propertyGrid";
  import currentResource from "@/orchestrators/currentResource";
  import log from "@lgn/web-client/src/lib/log";
  import { createEventDispatcher } from "svelte";
  import BooleanProperty from "./properties/BooleanProperty.svelte";
  import ColorProperty from "./properties/ColorProperty.svelte";
  import NumberProperty from "./properties/NumberProperty.svelte";
  import QuatProperty from "./properties/QuatProperty.svelte";
  import ScriptProperty from "./properties/ScriptProperty.svelte";
  import SpeedProperty from "./properties/SpeedProperty.svelte";
  import StringProperty from "./properties/StringProperty.svelte";
  import Vec3Property from "./properties/Vec3Property.svelte";
  import ResourcePathIdProperty from "./properties/ResourcePathIdProperty.svelte";
  import EnumProperty from "./properties/EnumProperty.svelte";
  import PropertyInputOption from "./PropertyInputOption.svelte";
  import type { RemoveVectorSubPropertyEvent } from "./types";

  const dispatch = createEventDispatcher<{
    input: PropertyUpdate;
    removeVectorSubProperty: RemoveVectorSubPropertyEvent;
  }>();

  const { data: currentResourceData } = currentResource;

  export let property: ResourceProperty;

  export let parentProperty: BagResourceProperty | null;

  /** The property path parts */
  export let pathParts: string[];

  /** The property index (only used in vectors) */
  export let index: number;

  function onInput({ value }: Pick<PropertyUpdate, "value">) {
    dispatch("input", {
      name: pathParts.join("."),
      value,
    });
  }

  function isReadonly(): boolean {
    return "readonly" in property.attributes;
  }

  // Vector related code
  // TODO: Extract this to a vector sub properties component?

  function removeVectorSubProperty() {
    if (!parentProperty) {
      log.error("Vector sub property parent not found");

      return;
    }

    if (!$currentResourceData) {
      log.error(
        "A vector sub property was removed while no resources were selected"
      );

      return;
    }

    parentProperty.subProperties = parentProperty.subProperties
      .filter(({ name }) => property.name !== name)
      .map((property, index) => ({
        ...property,
        name: `[${index}]`,
      }));

    dispatch("removeVectorSubProperty", {
      path: pathParts.slice(0, -1).join("."),
      index,
    });
  }
</script>

<div class="root">
  {#if propertyIsBoolean(property)}
    <div class="boolean-property">
      <BooleanProperty
        disabled={isReadonly()}
        on:input={({ detail }) => onInput({ value: detail })}
        bind:value={property.value}
      />
    </div>
  {:else if propertyIsScript(property)}
    <ScriptProperty
      readonly={isReadonly()}
      name={property.name}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsString(property)}
    <StringProperty
      readonly={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsResourcePathId(property)}
    <ResourcePathIdProperty
      readonly={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsEnum(property)}
    <EnumProperty
      disabled={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      value={{
        item: property.value,
        value: property.value,
      }}
      options={property.subProperties.map((variant) => ({
        item: variant.name,
        value: variant.name,
      }))}
    />
  {:else if propertyIsNumber(property)}
    <NumberProperty
      readonly={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsColor(property)}
    <ColorProperty
      readonly={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsSpeed(property)}
    <SpeedProperty
      readonly={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsVec3(property)}
    <Vec3Property
      readonly={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsQuat(property)}
    <QuatProperty
      readonly={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsVec(property)}
    {property.ptype} not implemented
  {:else if propertyIsOption(property)}
    <PropertyInputOption on:input {pathParts} {property} {index} />
  {:else}
    <div class="unknown-property">
      Unknown property: {property.ptype}
    </div>
  {/if}
  {#if parentProperty && propertyIsVec(parentProperty)}
    <div class="close-button" on:click={removeVectorSubProperty}>&#215;</div>
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row h-full w-full items-center;
  }

  .boolean-property {
    @apply flex flex-row w-full justify-end;
  }

  .unknown-property {
    @apply break-all;
  }

  .close-button {
    @apply flex flex-row flex-shrink-0 items-center justify-center h-6 w-6 rounded-sm text-xl bg-gray-800 ml-1 cursor-pointer;
  }
</style>
