<script lang="ts">
  import { colorSetFromHex } from "@/lib/colors";
  import clickOutside from "@/actions/clickOutside";

  import ColorPicker from "../ColorPicker.svelte";
  import TextInput from "../TextInput.svelte";

  export let value: string;

  let visible = false;
  let colors = colorSetFromHex(value);

  const setColors = (event: Event) => {
    if (event.currentTarget instanceof HTMLInputElement) {
      colors = colorSetFromHex(event.currentTarget.value);
    }
  };
</script>

<div
  class="root"
  use:clickOutside={() => {
    visible = false;
  }}
>
  <TextInput
    size="sm"
    value={colors.hex}
    on:input={setColors}
    fullWidth
    autoSelect
  >
    <ColorPicker slot="extension" bind:colors bind:visible position="left" />
  </TextInput>
</div>

<style lang="postcss">
  .root {
    @apply w-full;
  }
</style>
