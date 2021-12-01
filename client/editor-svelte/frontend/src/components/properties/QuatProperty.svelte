<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import NumberInput from "../NumberInput.svelte";

  type Quat = [number, number, number, number];

  const dispatch = createEventDispatcher<{ input: Quat }>();

  export let value: Quat;

  // TODO: Cleanup
  function updateVectorAt(
    index: 0 | 1 | 2 | 3,
    { detail }: CustomEvent<number>
  ) {
    switch (index) {
      case 0:
        return dispatch("input", [detail, value[1], value[2], value[3]]);
      case 1:
        return dispatch("input", [value[0], detail, value[2], value[3]]);
      case 2:
        return dispatch("input", [value[0], value[1], detail, value[3]]);
      case 3:
        return dispatch("input", [value[0], value[1], value[2], detail]);
    }
  }
</script>

<div class="root">
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(0, event)}
      bind:value={value[0]}
      noArrow
      fullWidth
      autoSelect
    />
  </div>
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(1, event)}
      bind:value={value[1]}
      noArrow
      fullWidth
      autoSelect
    />
  </div>
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(2, event)}
      bind:value={value[2]}
      noArrow
      fullWidth
      autoSelect
    />
  </div>
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(3, event)}
      bind:value={value[3]}
      noArrow
      fullWidth
      autoSelect
    />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row space-x-1;
  }
</style>
