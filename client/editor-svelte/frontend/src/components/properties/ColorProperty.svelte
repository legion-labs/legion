<script lang="ts">
  import TextInput from "../TextInput.svelte";

  export let value: string;
</script>

<div class="root">
  <TextInput bind:value fullWidth>
    <input slot="extension" class="color-picker" type="color" bind:value />
  </TextInput>
</div>

<style lang="postcss">
  .root {
    @apply w-full;
  }

  .color-picker {
    @apply h-full w-full cursor-pointer;
  }
</style>
