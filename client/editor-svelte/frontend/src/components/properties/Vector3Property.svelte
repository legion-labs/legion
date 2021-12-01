<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import NumberInput from "../NumberInput.svelte";

  type Vector3 = [number, number, number];

  const dispatch = createEventDispatcher<{ input: Vector3 }>();

  export let value: Vector3;

  // TODO: Cleanup
  function updateVectorAt(index: 0 | 1 | 2, { detail }: CustomEvent<number>) {
    switch (index) {
      case 0:
        return dispatch("input", [detail, value[1], value[2]]);
      case 1:
        return dispatch("input", [value[0], detail, value[2]]);
      case 2:
        return dispatch("input", [value[0], value[1], detail]);
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
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row space-x-1;
  }
</style>
