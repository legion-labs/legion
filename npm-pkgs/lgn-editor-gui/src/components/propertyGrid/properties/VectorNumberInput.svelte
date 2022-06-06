<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import NumberInput from "@/components/inputs/NumberInput.svelte";

  const dispatch = createEventDispatcher<{ input: number }>();

  type VectorInputType = "X" | "Y" | "Z" | "W";

  export let value: number;
  export let kind: VectorInputType;
  export let readonly = false;
</script>

<div class="vector-value">
  <div
    class="vector-value-name"
    class:bg-vector-x={kind === "X"}
    class:bg-vector-y={kind === "Y"}
    class:bg-vector-w={kind === "W"}
    class:bg-vector-z={kind === "Z"}
  />
  <NumberInput
    on:input={(event) => dispatch("input", event.detail)}
    bind:value
    noArrow={true}
    fluid={true}
    autoSelect
    {readonly}
  />
</div>

<style lang="postcss">
  .vector-value {
    @apply flex rounded-sm w-20;
  }

  .vector-value-name {
    @apply text-center text-xs my-auto w-[1px] h-5 rounded-l-sm;
  }
</style>
