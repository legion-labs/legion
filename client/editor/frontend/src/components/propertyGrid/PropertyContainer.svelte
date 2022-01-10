<script lang="ts">
  import {
    propertyIsComponent,
    propertyIsGroup,
    propertyIsOption,
    ResourceProperty,
  } from "@/api/propertyGrid";
  import GroupProperty from "./GroupProperty.svelte";
  import Property from "./Property.svelte";

  export let property: ResourceProperty;

  export let level = 0;

  /** The property path parts */
  export let pathParts: string[];

  /** Displays a nice little border below the resource property (or not)! */
  export let withBorder: boolean;
</script>

<div class="root">
  {#if propertyIsGroup(property)}
    <GroupProperty on:input {property} {level} {pathParts} {withBorder} />
  {:else if propertyIsComponent(property)}
    <GroupProperty on:input {property} {level} {pathParts} {withBorder} />
  {:else if propertyIsOption(property) && property.subProperties[0] && (propertyIsGroup(property.subProperties[0]) || propertyIsComponent(property.subProperties[0]))}
    <GroupProperty on:input {property} {level} {pathParts} {withBorder} />
  {:else}
    <Property on:input {property} {pathParts} {withBorder} />
  {/if}
</div>

<style lang="postcss">
  .root {
    @apply last:pb-1;
  }
</style>
