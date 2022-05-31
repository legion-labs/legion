<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { Vec3 } from "@/components/propertyGrid/lib/propertyGrid";

  import VectorNumberInput from "./VectorNumberInput.svelte";

  const dispatch = createEventDispatcher<{ input: Vec3 }>();

  export let value: Vec3;

  export let readonly = false;

  function updateVectorAt(index: 0 | 1 | 2, { detail }: CustomEvent<number>) {
    dispatch("input", Object.assign([], value, { [index]: detail }));
  }
</script>

<div class="vector-root">
  <VectorNumberInput
    kind="X"
    bind:value={value[0]}
    on:input={(event) => updateVectorAt(0, event)}
    {readonly}
  />
  <VectorNumberInput
    kind="Y"
    bind:value={value[1]}
    on:input={(event) => updateVectorAt(1, event)}
    {readonly}
  />
  <VectorNumberInput
    kind="Z"
    bind:value={value[2]}
    on:input={(event) => updateVectorAt(2, event)}
    {readonly}
  />
</div>

<style lang="postcss">
  .vector-root {
    @apply flex gap-x-2 my-auto;
  }
</style>
