<script lang="ts">
  import NumberInput from "../NumberInput.svelte";

  export let value: number;
</script>

<!-- For now the number property is only a NumberInput but it might change -->
<NumberInput on:input bind:value noArrow fullWidth autoSelect />
