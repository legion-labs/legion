<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import NumberInput from "@/components/inputs/NumberInput.svelte";

  const dispatch = createEventDispatcher<{ input: number }>();

  type VectorInputType = "X" | "Y" | "Z" | "W";

  export let value: number;
  export let readonly = false;
  export let kind: VectorInputType;
</script>

<div class="flex rounded-sm">
  <div
    class:bg-red-900={kind === "X"}
    class:bg-green-900={kind === "Y"}
    class:bg-blue-900={kind === "Z"}
    class:bg-yellow-900={kind === "W"}
    class="h-8 w-8 text-center"
  >
    {kind}
  </div>
  <div class="bg-surface-900">
    <NumberInput
      on:input={(event) => dispatch("input", event.detail)}
      bind:value
      noArrow
      fluid
      autoSelect
      {readonly}
    />
  </div>
</div>
