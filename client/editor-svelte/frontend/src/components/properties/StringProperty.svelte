<script lang="ts">
  import TextInput from "../TextInput.svelte";

  export let value: string;
</script>

<!-- For now the string property is only a TextInput but it might change -->
<TextInput bind:value size="sm" fullWidth />
