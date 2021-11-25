<script lang="ts">
  import Toggle from "../Toggle.svelte";

  export let value: boolean;
</script>

<!-- For now the boolean property is only a Toggle but it might change -->
<Toggle bind:value />
