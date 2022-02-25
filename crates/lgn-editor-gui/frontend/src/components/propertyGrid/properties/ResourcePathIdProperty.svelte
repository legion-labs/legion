<script lang="ts">
  import TextInput from "../../inputs/TextInput.svelte";

  export let value: string;

  export let readonly = false;
</script>

<!-- For now the string property is only a TextInput but it might change -->
<TextInput on:input bind:value fluid autoSelect {readonly} />
