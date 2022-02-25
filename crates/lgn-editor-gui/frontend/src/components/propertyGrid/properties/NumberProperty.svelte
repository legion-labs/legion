<script lang="ts">
  import NumberInput from "../../inputs/NumberInput.svelte";

  export let value: number;

  export let readonly = false;
</script>

<!-- For now the number property is only a NumberInput but it might change -->
<NumberInput
  align="right"
  on:input
  bind:value
  noArrow
  fluid
  autoSelect
  {readonly}
/>
