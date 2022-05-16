<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { Quat } from "@/lib/propertyGrid";

  import VectorNumberInput from "./VectorNumberInput.svelte";

  const dispatch = createEventDispatcher<{ input: Quat }>();

  export let value: Quat;

  export let readonly = false;

  function updateVectorAt(
    index: 0 | 1 | 2 | 3,
    { detail }: CustomEvent<number>
  ) {
    dispatch("input", Object.assign([], value, { [index]: detail }));
  }
</script>

<div class="quaternion-root">
  <VectorNumberInput
    kind="W"
    bind:value={value[3]}
    on:input={(event) => updateVectorAt(3, event)}
    {readonly}
  />
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
  .quaternion-root {
    @apply flex justify-end gap-x-2 my-auto;
  }
</style>
