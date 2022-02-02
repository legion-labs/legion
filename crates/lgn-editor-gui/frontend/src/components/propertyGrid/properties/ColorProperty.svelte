<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import clickOutside from "@lgn/web-client/src/actions/clickOutside";
  import { ColorSet, colorSetFromHex } from "@/lib/colors";
  import ColorPicker from "@/components/inputs/ColorPicker.svelte";
  import TextInput from "@/components/inputs/TextInput.svelte";

  const dispatch = createEventDispatcher<{
    input: number;
  }>();

  export let value: number;

  export let disabled = false;

  let visible = false;

  $: hexValue = value.toString(16).padStart(8, "0");

  function setColors(newValue: string) {
    value = parseInt(newValue, 16);

    dispatch("input", value);
  }

  function setColorsFromTextInput({ detail: hex }: CustomEvent<string>) {
    setColors(hex);
  }

  function setColorsFromColorPicker({
    detail: { hex },
  }: CustomEvent<ColorSet>) {
    setColors(hex);
  }

  function hideDropdown() {
    visible = false;
  }
</script>

<div class="root" use:clickOutside on:click-outside={hideDropdown}>
  <TextInput
    value={hexValue}
    on:input={setColorsFromTextInput}
    fullWidth
    autoSelect
    {disabled}
  >
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
      colors={colorSetFromHex(hexValue)}
      position="left"
      {disabled}
    />
  </TextInput>
</div>

<style lang="postcss">
  .root {
    @apply w-full;
  }
</style>
