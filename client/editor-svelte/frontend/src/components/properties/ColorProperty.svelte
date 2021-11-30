<script lang="ts">
  import { ColorSet, colorSetFromHex } from "@/lib/colors";
  import clickOutside from "@/actions/clickOutside";

  import ColorPicker from "../ColorPicker.svelte";
  import TextInput from "../TextInput.svelte";

  export let value: string;

  let visible = false;

  const setColorsFromTextInput = ({
    detail: newValue,
  }: CustomEvent<string>) => {
    value = newValue;
  };

  const setColorsFromColorPicker = ({
    detail: { hex },
  }: CustomEvent<ColorSet>) => {
    value = hex;
  };
</script>

<div
  class="root"
  use:clickOutside={() => {
    visible = false;
  }}
>
  <TextInput {value} on:input={setColorsFromTextInput} fullWidth autoSelect>
    <ColorPicker
      slot="extension"
      on:change={setColorsFromColorPicker}
      bind:visible
      colors={colorSetFromHex(value)}
      position="left"
    />
  </TextInput>
</div>

<style lang="postcss">
  .root {
    @apply w-full;
  }
</style>
