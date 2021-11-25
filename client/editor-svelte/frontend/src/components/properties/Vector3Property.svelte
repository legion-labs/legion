<script lang="ts">
  import NumberInput from "../NumberInput.svelte";

  export let value: [number, number, number];
</script>

<div class="root">
  <div>
    <NumberInput bind:value={value[0]} size="sm" noArrow fullWidth />
  </div>
  <div>
    <NumberInput bind:value={value[1]} size="sm" noArrow fullWidth />
  </div>
  <div>
    <NumberInput bind:value={value[2]} size="sm" noArrow fullWidth />
  </div>
</div>

<style lang="postcss">
  .root {
    @apply flex flex-row space-x-1;
  }
</style>
