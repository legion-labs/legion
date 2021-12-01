<script lang="ts">
  import Range from "../Range.svelte";

  export let value: number;
</script>

<!-- For now the number property is only a Range but it might change -->
<Range min={-10} max={10} fullWidth withNumberInput bind:value />
