<script lang="ts">
  import Toggle from "../../inputs/Toggle.svelte";

  export let value: boolean;

  export let disabled = false;
</script>

<!-- For now the boolean property is only a Toggle but it might change -->
<Toggle on:input bind:value {disabled} />
