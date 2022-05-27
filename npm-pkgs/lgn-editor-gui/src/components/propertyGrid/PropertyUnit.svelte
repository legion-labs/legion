<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { Writable } from "svelte/store";

  import HighlightedText from "@lgn/web-client/src/components/HighlightedText.svelte";
  import { stringToSafeRegExp } from "@lgn/web-client/src/lib/html";

  import type { PropertyUpdate } from "@/api";
  import {
    isPropertyDisplayable,
    propertyIsVec,
  } from "@/components/propertyGrid/lib/propertyGrid";
  import type {
    BagResourceProperty,
    ResourceProperty,
  } from "@/components/propertyGrid/lib/propertyGrid";

  import PropertyInput from "./PropertyInput.svelte";
  import type { RemoveVectorSubPropertyEvent } from "./types";

  type $$Events = {
    input: CustomEvent<PropertyUpdate>;
    removeVectorSubProperty: CustomEvent<RemoveVectorSubPropertyEvent>;
    displayable: CustomEvent<boolean>;
  };

  const dispatch = createEventDispatcher<{
    displayable: boolean;
  }>();

  export let property: ResourceProperty;

  export let parentProperty: BagResourceProperty | null;

  /** The property path parts */
  export let pathParts: string[];

  /** The property index (only used in vectors) */
  export let index: number;

  export let level: number;

  export let search: Writable<string>;

  let displayable = true;

  $: if (parentProperty && !propertyIsVec(parentProperty)) {
    displayable = isPropertyDisplayable(property.name, $search);
    dispatch("displayable", displayable);
  }

  function beautifyPropertyName(name: string) {
    const split = name.split("_");

    for (let i = 0; i < split.length; i++) {
      split[i] = split[i][0].toUpperCase() + split[i].slice(1, split[i].length);
    }

    return split.join(" ");
  }
</script>

{#if displayable}
  <div
    class="property-unit-root"
    style="padding-left:{level / 4}rem"
    class:bg-surface-700={index % 2 === 0}
    class:bg-surface-800={index % 2 !== 0}
  >
    {#if property.name}
      <div class="property-name" title={`${property.name} (${property.ptype})`}>
        {#if search}
          <HighlightedText
            pattern={stringToSafeRegExp($search, "gi")}
            text={beautifyPropertyName(property.name)}
          />
        {:else}
          {beautifyPropertyName(property.name)}
        {/if}
      </div>
    {/if}
    <div class="property-input-container">
      <div class="property-input">
        <PropertyInput
          on:input
          on:removeVectorSubProperty
          pathParts={property.name ? [...pathParts, property.name] : pathParts}
          {property}
          {index}
          bind:parentProperty
        />
      </div>
    </div>
  </div>
{/if}

<style lang="postcss">
  .property-unit-root {
    @apply flex flex-row justify-between h-9 pr-2;
  }

  .property-name {
    @apply my-auto truncate text-item-mid;
  }

  .property-input-container {
    @apply flex w-[10rem] flex-shrink-0 flex-grow-[0.5];
  }

  .property-input {
    @apply flex w-full justify-end;
  }
</style>
