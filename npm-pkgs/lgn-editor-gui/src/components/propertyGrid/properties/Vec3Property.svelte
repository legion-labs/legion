<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { Vec3 } from "@/lib/propertyGrid";

  import NumberInput from "../../inputs/NumberInput.svelte";
  import VectorNumberInput from "./VectorNumberInput.svelte";

  const dispatch = createEventDispatcher<{ input: Vec3 }>();

  export let value: Vec3;

  export let readonly = false;

  function updateVectorAt(index: 0 | 1 | 2, { detail }: CustomEvent<number>) {
    dispatch("input", Object.assign([], value, { [index]: detail }));
  }
</script>

<div class="flex flex-row space-x-1">
  <VectorNumberInput
    kind="X"
    bind:value={value[0]}
    on:input={(event) => updateVectorAt(0, event)}
  />
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(0, event)}
      bind:value={value[0]}
      noArrow
      fluid
      autoSelect
      {readonly}
    />
  </div>
  <VectorNumberInput
    kind="Y"
    bind:value={value[1]}
    on:input={(event) => updateVectorAt(1, event)}
  />
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(1, event)}
      bind:value={value[1]}
      noArrow
      fluid
      autoSelect
      {readonly}
    />
  </div>
  <VectorNumberInput
    kind="Z"
    bind:value={value[2]}
    on:input={(event) => updateVectorAt(2, event)}
  />
  <div>
    <NumberInput
      on:input={(event) => updateVectorAt(2, event)}
      bind:value={value[2]}
      noArrow
      fluid
      autoSelect
      {readonly}
    />
  </div>
</div>
