<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { PropertyUpdate } from "@/api";
  import {
    buildDefaultPrimitiveProperty,
    extractOptionPType,
    propertyIsOption,
    ptypeBelongsToPrimitive,
  } from "@/components/propertyGrid/lib/propertyGrid";
  import type { OptionResourceProperty } from "@/components/propertyGrid/lib/propertyGrid";

  import PropertyInput from "./PropertyInput.svelte";
  import PropertyActionButton from "./PropertyActionButton.svelte";

  const dispatch = createEventDispatcher<{
    input: PropertyUpdate;
  }>();

  export let property: OptionResourceProperty;

  /** The property path parts */
  export let pathParts: string[];

  /** The property index (only used in vectors) */
  export let index: number;

  function setOptionProperty(optionEnabled: boolean) {
    // TODO: Send an input event that be can sent to the server

    // Not supposed to happen, we can consider casting
    // the property as an option resource property at that point
    if (!propertyIsOption(property)) {
      return;
    }

    if (optionEnabled) {
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
    <PropertyActionButton
      icon="ic:baseline-subdirectory-arrow-left"
      on:click={(_) => setOptionProperty(false)}
    />
  </div>
{:else}
  <PropertyActionButton
    icon="ic:baseline-add-circle-outline"
    on:click={(_) => setOptionProperty(true)}
  />
{/if}

<style lang="postcss">
  .option-property {
    @apply flex flex-row justify-between gap-x-1;
  }
</style>
