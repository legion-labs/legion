<script lang="ts">
  import Range from "../../inputs/Range.svelte";

  export let value: number;

  export let readonly = false;
</script>

<!-- For now the number property is only a Range but it might change -->
<Range
  min={-10}
  max={10}
  fluid
  withNumberInput
  on:input
  bind:value
  {readonly}
/>
