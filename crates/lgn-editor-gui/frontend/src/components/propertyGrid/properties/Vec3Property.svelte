<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { Vec3 } from "@/lib/propertyGrid";
  import NumberInput from "../../inputs/NumberInput.svelte";

  const dispatch = createEventDispatcher<{ input: Vec3 }>();

  export let value: Vec3;

  export let disabled = false;

  function updateVectorAt(index: 0 | 1 | 2, { detail }: CustomEvent<number>) {
    dispatch("input", Object.assign([], value, { [index]: detail }));
  }
</script>

<div class="root">
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(0, event)}
      bind:value={value[0]}
      noArrow
      fluid
      autoSelect
      {disabled}
    />
  </div>
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(1, event)}
      bind:value={value[1]}
      noArrow
      fluid
      autoSelect
      {disabled}
    />
  </div>
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(2, event)}
      bind:value={value[2]}
      noArrow
      fluid
      autoSelect
      {disabled}
    />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row space-x-1;
  }
</style>
