<script lang="ts">
  import type { PropertyUpdate } from "@/api";
  import {
    buildDefaultPrimitiveProperty,
    extractOptionPType,
    propertyIsOption,
    ptypeBelongsToPrimitive,
  } from "@/lib/propertyGrid";
  import type { OptionResourceProperty } from "@/lib/propertyGrid";
  import { createEventDispatcher } from "svelte";
  import Checkbox from "../inputs/Checkbox.svelte";
  import PropertyInput from "./PropertyInput.svelte";

  const dispatch = createEventDispatcher<{
    input: PropertyUpdate;
  }>();

  export let property: OptionResourceProperty;

  /** The property path parts */
  export let pathParts: string[];

  /** The property index (only used in vectors) */
  export let index: number;

  function setOptionProperty({ detail: isSome }: CustomEvent<boolean>) {
    // TODO: Send an input event that be can sent to the server

    // Not supposed to happen, we can consider casting
    // the property as an option resource property at that point
    if (!propertyIsOption(property)) {
      return;
    }

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
      property.subProperties = [];

      dispatch("input", {
        name: pathParts.join("."),
        value: null,
      });
    }
  }
</script>

{#if property.subProperties[0]}
  <div class="option-property">
    <PropertyInput
      on:input={(event) => dispatch("input", event.detail)}
      property={property.subProperties[0]}
      parentProperty={property}
      {pathParts}
      {index}
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
      <div class="cursor-help-icon">?</div>
    </div>
    <div class="option-property-checkbox">
      <Checkbox on:change={setOptionProperty} value={false} />
    </div>
  </div>
{/if}

<style lang="postcss">
  .option-property {
    @apply flex flex-row justify-between h-full w-full;
  }

  .option-property-checkbox {
    @apply flex items-center flex-shrink-0 h-full pl-1;
  }

  .cursor-help {
    @apply flex flex-row h-8 pt-1;
  }

  .cursor-help-icon {
    @apply flex flex-row self-start justify-center items-center text-xs h-4 w-4 bg-gray-500 rounded-full;
  }
</style>
