<!--
@Component
Simple color picker component.

It supports HSV edition via 2 different visual inputs
one to set the Hue (a simple slider)
and another one to set both the Saturation and the Value.

A slider is also provided to allow for alpha channel edition.

It also supports manual RGBA edition with 4 different inputs.
-->
<script lang="ts">
  // TODO: We could split this component into several components (Hue, SaturationValue, RGBA, etc...)

  import colorConvert from "color-convert";
  import {
    ColorSet,
    colorSetFromHsv,
    colorSetFromRgba,
    hsvToColorString,
    maxHueValue,
    Rgba,
    rgbaToColorString,
  } from "@/lib/colors";
  import NumberInput from "./NumberInput.svelte";

  // TODO: Use a better/smaller representation instead of ColorSet to prevent constent data conversion
  /** The colors props is a `ColorSet`, that is, a combination of 3 different color
   * representations: [HSV](https://en.wikipedia.org/wiki/HSL_and_HSV),
   * [RGBA](https://en.wikipedia.org/wiki/RGBA_color_model), and hex.
   * The reason is that converting from HSV to RGB/Hex and back is lossy
   * and can lead to glitch with the UI.
   *
   * For exemple converting any grey "color" like `120° 0% 20%` to the RGB equivalent `rgb(51, 51, 51)`
   * and back to HSV will return `0° 0% 20%` where the Hue will always be `0°`.
   *
   * What it means in practice is that we need to convert HSV <-> RGBA <-> hex
   * on _each_ color change, input typing, etc... The conversion is fast so far and no performance
   * issues are to be expected any time soon but it's something we might need to change at one point.
   */
  export let colors: ColorSet;
  /** Position of the `ColorPicker` drowdown */
  export let position: "left" | "right" = "right";
  /** Show the `ColorPicker` or not */
  export let visible = false;

  /** A word on semantic:
   * `hPicker` refers to the Hue range input
   * `svPicker` to the "main" block that allows for both Saturation and Value selection
   * `aPicker` refers to the Alpha channel range input
   */

  /** The Saturation and Value picker width */
  let svPickerCursorWidth: number | undefined;
  /** The Saturation and Value picker height */
  let svPickerCursorHeight: number | undefined;
  /** When `true` indicates the user is moving their mouse over the Saturation and Value picker */
  let svPickerDragging = false;
  /** Left position of the picker cursor over the Saturation and Value picker */
  let svPickerLeft = 0;
  /** Top position of the picker cursor over the Saturation and Value picker */
  let svPickerTop = 0;
  /** The currently seleted Hue value as an Rgba color */
  let hColor: Rgba;

  // Sets the Saturation and Value picker selector position on color change
  $: if (svPickerCursorWidth && svPickerCursorHeight) {
    svPickerLeft = (svPickerCursorWidth / 100) * colors.hsv.s;
    svPickerTop = (svPickerCursorHeight / 100) * (100 - colors.hsv.v);
  }

  // Sets the Saturation and Value picker background color on Hue change
  $: {
    const [r, g, b] = colorConvert.hsv.rgb([colors.hsv.h, 100, 100]);

    hColor = { r, g, b, a: 1 };
  }

  /** Called when the user changes the Saturation and Value */
  function svSelect(
    event: MouseEvent & { currentTarget: EventTarget & HTMLDivElement }
  ) {
    svPickerDragging = true;

    const xPercentage = (100 / event.currentTarget.offsetWidth) * event.offsetX;
    const yPercentage =
      (100 / event.currentTarget.offsetHeight) * event.offsetY;

    colors = colorSetFromHsv({
      h: colors.hsv.h,
      s: xPercentage,
      v: 100 - yPercentage,
      alpha: colors.hsv.alpha,
    });
  }

  function updateHue(
    event: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    colors = colorSetFromHsv({
      ...colors.hsv,
      h: +event.currentTarget.value,
    });
  }

  function updateHsvAlpha(
    event: Event & { currentTarget: EventTarget & HTMLInputElement }
  ) {
    colors = colorSetFromHsv({
      ...colors.hsv,
      alpha: +event.currentTarget.value,
    });
  }

  function updateRgbaColor(key: keyof Rgba) {
    return (event: Event) => {
      const isAlpha = key === "a";

      const newColorPart = +(
        (event.currentTarget as HTMLInputElement | undefined)?.value ??
        colors.rgba[key]
      );

      if (
        newColorPart >= 0 &&
        ((isAlpha && newColorPart <= 1) || (!isAlpha && newColorPart <= 255))
      ) {
        colors = colorSetFromRgba({
          ...colors.rgba,
          [key]: newColorPart,
        });
      }
    };
  }

  function svSelectMove(
    event: MouseEvent & { currentTarget: EventTarget & HTMLDivElement }
  ) {
    if (svPickerDragging) {
      svSelect(event);
    }
  }

  /** Called when the user is no longer changin Saturation and Value */
  function svSelectEnd() {
    svPickerDragging = false;
  }

  function toggle() {
    visible = !visible;
  }
</script>

<div class="root">
  <div
    class="dropdown-toggle"
    on:click={toggle}
    style="--current-rgba-color: {hsvToColorString(colors.hsv)}"
  />
  <div
    class="dropdown"
    class:visible
    class:invisible={!visible}
    class:right-0={position === "left"}
  >
    <div class="sv-selector-input">
      <div
        class="sv-selector-background"
        style="--current-background: {rgbaToColorString(hColor)}"
      >
        <div class="sv-selector-white-gradient-mask">
          <div
            class="sv-selector-black-gradient-mask"
            bind:clientWidth={svPickerCursorWidth}
            bind:clientHeight={svPickerCursorHeight}
            on:mousedown={svSelect}
            on:mouseup={svSelectEnd}
            on:mousemove={svSelectMove}
            on:mouseleave={svSelectEnd}
          />
          <div
            class="sv-selector-cursor"
            style="--color-block-top: {`${
              svPickerTop - 6
            }px`}; --color-block-left: {`${
              svPickerLeft - 6
            }px`}; --current-rgba-color: {hsvToColorString(colors.hsv, true)}"
          />
        </div>
      </div>
    </div>
    <div class="additional-selectors">
      <div class="h-selector-container">
        <input
          type="range"
          min={0}
          max={maxHueValue}
          class="h-selector"
          style="--current-background: {rgbaToColorString(hColor, true)}"
          value={colors.hsv.h}
          on:input={updateHue}
        />
      </div>
      <div class="alpha-selector-container">
        <div class="alpha-selector-checkered-mask">
          <div
            class="alpha-selector-opacity-mask"
            style="--tw-gradient-to: {rgbaToColorString(hColor, true)}"
          >
            <input
              type="range"
              min={0}
              max={100}
              class="alpha-selector"
              style="--current-background: {rgbaToColorString(hColor, true)}"
              value={colors.hsv.alpha}
              on:input={updateHsvAlpha}
            />
          </div>
        </div>
      </div>
      <div class="rgba-inputs">
        <div>
          <NumberInput
            autoSelect
            noArrow
            fullWidth
            size="sm"
            min={0}
            max={255}
            value={colors.rgba.r}
            on:input={updateRgbaColor("r")}
          />
        </div>
        <div>
          <NumberInput
            autoSelect
            noArrow
            fullWidth
            size="sm"
            min={0}
            max={255}
            value={colors.rgba.g}
            on:input={updateRgbaColor("g")}
          />
        </div>
        <div>
          <NumberInput
            autoSelect
            noArrow
            fullWidth
            size="sm"
            min={0}
            max={255}
            value={colors.rgba.b}
            on:input={updateRgbaColor("b")}
          />
        </div>
        <div>
          <NumberInput
            autoSelect
            noArrow
            fullWidth
            size="sm"
            min={0}
            max={1}
            step={0.01}
            value={colors.rgba.a}
            on:input={updateRgbaColor("a")}
          />
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .root {
    @apply relative h-full w-full;
  }

  .dropdown-toggle {
    @apply h-full w-full border border-white cursor-pointer;
    background-color: var(--current-rgba-color);
  }

  .dropdown {
    @apply flex flex-col w-48 border border-gray-800 absolute bg-gray-700 rounded-b-sm mt-1 shadow-xl;
  }

  .sv-selector-input {
    @apply flex flex-col w-full rounded-sm space-y-1;
  }

  .sv-selector-background {
    @apply h-48 w-full relative;
    background-color: var(--current-background);
  }

  .sv-selector-white-gradient-mask {
    @apply h-full w-full bg-gradient-to-r from-white to-transparent;
  }

  .sv-selector-black-gradient-mask {
    @apply h-full w-full bg-gradient-to-b from-transparent to-black;
  }

  .sv-selector-cursor {
    @apply h-4 w-4 rounded-full border-2 border-gray-700 absolute pointer-events-none;
    top: var(--color-block-top);
    left: var(--color-block-left);
    background: var(--current-rgba-color);
  }

  .additional-selectors {
    @apply flex flex-col p-2 rounded-b-sm space-y-2;
  }

  .h-selector-container {
    @apply flex items-center h-4 w-full;
  }

  .h-selector {
    @apply h-2 border-none rounded-full w-full appearance-none;
    background: linear-gradient(
      to right,
      #ff0000 0%,
      #ffff00 17%,
      #00ff00 33%,
      #00ffff 50%,
      #0000ff 67%,
      #ff00ff 83%,
      #ff0000 100%
    );
  }

  .h-selector::-moz-range-thumb {
    @apply w-3 h-3 cursor-pointer border-2 border-gray-700 rounded-full;
    background-color: var(--current-background);
  }

  .h-selector::-webkit-slider-thumb {
    @apply bg-gray-800 w-4 h-4 cursor-pointer border-2 border-gray-700 rounded-full appearance-none;
  }

  .alpha-selector-container {
    @apply flex items-center h-4 w-full;
  }

  .alpha-selector-checkered-mask {
    @apply w-full h-2 rounded-full;
    background: repeating-conic-gradient(
        theme("colors.gray.400") 0deg 90deg,
        theme("colors.gray.700") 0deg 180deg
      )
      0 0 / theme("spacing.2");
  }

  .alpha-selector-opacity-mask {
    @apply w-full h-full relative rounded-full bg-gradient-to-r from-transparent;
  }

  .alpha-selector {
    @apply bg-transparent h-2 absolute border-none rounded-full w-full appearance-none;
  }

  .alpha-selector::-moz-range-thumb {
    @apply w-3 h-3 cursor-pointer border-2 border-gray-700 rounded-full;
    background-color: var(--current-background);
  }

  .alpha-selector::-webkit-slider-thumb {
    @apply bg-gray-800 w-4 h-4 cursor-pointer border-2 border-gray-700 rounded-full appearance-none;
  }

  .rgba-inputs {
    @apply flex flex-row bg-gray-700 space-x-0.5;
  }
</style>
