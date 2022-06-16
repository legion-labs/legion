<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import log from "@lgn/web-client/src/lib/log";

  import type { PropertyUpdate } from "@/api";
  import {
    getResourceType,
    propertyIsBoolean,
    propertyIsColor,
    propertyIsEnum,
    propertyIsNumber,
    propertyIsOption,
    propertyIsQuat,
    propertyIsResourcePathId,
    propertyIsScript,
    propertyIsSpeed,
    propertyIsString,
    propertyIsVec,
    propertyIsVec3,
  } from "@/components/propertyGrid/lib/propertyGrid";
  import type {
    BagResourceProperty,
    ResourceProperty,
  } from "@/components/propertyGrid/lib/propertyGrid";
  import { currentResource } from "@/orchestrators/currentResource";

  import PropertyInputOption from "./PropertyInputOption.svelte";
  import BooleanProperty from "./properties/BooleanProperty.svelte";
  import ColorProperty from "./properties/ColorProperty.svelte";
  import EnumProperty from "./properties/EnumProperty.svelte";
  import NumberProperty from "./properties/NumberProperty.svelte";
  import QuatProperty from "./properties/QuatProperty.svelte";
  import ResourcePathIdProperty from "./properties/ResourcePathIdProperty.svelte";
  import ScriptProperty from "./properties/ScriptProperty.svelte";
  import SpeedProperty from "./properties/SpeedProperty.svelte";
  import StringProperty from "./properties/StringProperty.svelte";
  import Vec3Property from "./properties/Vec3Property.svelte";
  import type { RemoveVectorSubPropertyEvent } from "./types";

  const dispatch = createEventDispatcher<{
    input: PropertyUpdate;
    removeVectorSubProperty: RemoveVectorSubPropertyEvent;
  }>();

  export let property: ResourceProperty;

  export let parentProperty: BagResourceProperty | null;

  /** The property path parts */
  export let pathParts: string[];

  /** The property index (only used in vectors) */
  export let index: number;

  $: propertyType = getResourceType(property, parentProperty);

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

    if (!$currentResource) {
      log.error(
        "A vector sub property was removed while no resources were selected"
      );

      return;
    }

    // eslint-disable-next-line camelcase
    parentProperty.sub_properties = parentProperty.sub_properties
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

<div class="property-input-root">
  {#if propertyIsBoolean(property)}
    <BooleanProperty
      disabled={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsScript(property)}
    <ScriptProperty
      readonly={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
    />
  {:else if propertyIsString(property)}
    <div class="fixed-size">
      <StringProperty
        readonly={isReadonly()}
        on:input={({ detail }) => onInput({ value: detail })}
        bind:value={property.value}
      />
    </div>
  {:else if propertyIsResourcePathId(property)}
    <ResourcePathIdProperty
      readonly={isReadonly()}
      on:input={({ detail }) => onInput({ value: detail })}
      bind:value={property.value}
      resourceType={propertyType}
    />
  {:else if propertyIsEnum(property)}
    <div class="fixed-size">
      <EnumProperty
        disabled={isReadonly()}
        on:input={({ detail }) => onInput({ value: detail })}
        value={{
          item: property.value,
          value: property.value,
        }}
        options={property.sub_properties.map((variant) => ({
          item: variant.name,
          value: variant.name,
        }))}
      />
    </div>
  {:else if propertyIsNumber(property)}
    <div class="fixed-size">
      <NumberProperty
        readonly={isReadonly()}
        on:input={({ detail }) => onInput({ value: detail })}
        bind:value={property.value}
      />
    </div>
  {:else if propertyIsColor(property)}
    <div class="fixed-size">
      <ColorProperty
        readonly={isReadonly()}
        on:input={({ detail }) => onInput({ value: detail })}
        bind:value={property.value}
      />
    </div>
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
  .property-input-root {
    @apply text-item-mid;
    @apply flex justify-end;
    @apply w-full;
  }

  .fixed-size {
    @apply w-[16rem];
  }

  .unknown-property {
    @apply break-all;
  }

  .close-button {
    @apply flex flex-row items-center justify-center h-6 w-6 rounded-sm text-xl bg-surface-500 ml-1 cursor-pointer border border-black;
  }
</style>
