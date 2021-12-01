<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { ColorSet, colorSetFromHex } from "@/lib/colors";
  import clickOutside from "@/actions/clickOutside";
  import ColorPicker from "../ColorPicker.svelte";
  import TextInput from "../TextInput.svelte";

  const dispatch = createEventDispatcher<{
    input: string;
  }>();

  export let value: string;

  let visible = false;

  const setColorsFromTextInput = ({
    detail: newValue,
  }: CustomEvent<string>) => {
    value = newValue;
    dispatch("input", value);
  };

  const setColorsFromColorPicker = ({
    detail: { hex },
  }: CustomEvent<ColorSet>) => {
    value = hex;
    dispatch("input", value);
  };
</script>

<div
  class="root"
  use:clickOutside={() => {
    visible = false;
  }}
>
  <TextInput {value} on:input={setColorsFromTextInput} fullWidth autoSelect>
    <div
      class="h-full w-full flex items-center justify-center text-xl font-bold"
      slot="leftExtension"
      title="Hexadecimal color value"
    >
      #
    </div>
    <ColorPicker
      slot="rightExtension"
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
