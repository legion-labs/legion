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
  export let position: "left" | "right" = "right";
  export let visible = false;

  let colorBlockCursorWidth: number | undefined;
  let colorBlockCursorHeight: number | undefined;
  let colorBlockDragging = false;
  let colorBlockLeft = 0;
  let colorBlockTop = 0;

  $: if (colorBlockCursorWidth && colorBlockCursorHeight) {
    colorBlockLeft = (colorBlockCursorWidth / 100) * colors.hsv.s;
    colorBlockTop = (colorBlockCursorHeight / 100) * (100 - colors.hsv.v);
  }

  $: hColors = colorConvert.hsv.rgb([colors.hsv.h, 100, 100]);

  $: hColor = { r: hColors[0], g: hColors[1], b: hColors[2], a: 1 };

  function colorDrag(
    event: MouseEvent & { currentTarget: EventTarget & HTMLDivElement }
  ) {
    colorBlockDragging = true;

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

  function colorDragMove(
    event: MouseEvent & { currentTarget: EventTarget & HTMLDivElement }
  ) {
    if (colorBlockDragging) {
      colorDrag(event);
    }
  }

  function colorStopDragging() {
    colorBlockDragging = false;
  }

  function toggle() {
    visible = !visible;
  }

  $: console.log("xxx r", colors.rgba.r);
</script>

<div class="root">
  <div
    class="color"
    on:click={toggle}
    style="--current-rgba-color: {hsvToColorString(colors.hsv)}"
  />
  <div
    class="color-picker-dropdown"
    class:visible
    class:invisible={!visible}
    class:right-0={position === "left"}
  >
    <div class="color-picker-selector">
      <div
        class="color-block-background"
        style="--current-background: {rgbaToColorString(hColor)}"
      >
        <div class="color-block-white-gradient">
          <div
            class="color-block-black-gradient"
            bind:clientWidth={colorBlockCursorWidth}
            bind:clientHeight={colorBlockCursorHeight}
            on:mousedown={colorDrag}
            on:mouseup={colorStopDragging}
            on:mousemove={colorDragMove}
            on:mouseleave={colorStopDragging}
          />
          <div
            class="color-block-cursor"
            style="--color-block-top: {`${
              colorBlockTop - 6
            }px`}; --color-block-left: {`${
              colorBlockLeft - 6
            }px`}; --current-rgba-color: {hsvToColorString(colors.hsv, true)}"
          />
        </div>
      </div>
    </div>
    <div class="color-picker-extra-selectors">
      <div class="color-strip">
        <input
          type="range"
          min={0}
          max={maxHueValue}
          class="color-strip-selector"
          style="--current-background: {rgbaToColorString(hColor, true)}"
          value={colors.hsv.h}
          on:input={updateHue}
        />
      </div>
      <div class="alpha-strip">
        <div class="alpha-strip-checkered-mask">
          <div
            class="alpha-strip-opacity-mask"
            style="--tw-gradient-to: {rgbaToColorString(hColor, true)}"
          >
            <input
              type="range"
              min={0}
              max={100}
              class="alpha-strip-selector"
              style="--current-background: {rgbaToColorString(hColor, true)}"
              value={colors.hsv.alpha}
              on:input={updateHsvAlpha}
            />
          </div>
        </div>
      </div>
      <div class="inputs">
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

  .color {
    @apply h-full w-full border border-white cursor-pointer;
    background-color: var(--current-rgba-color);
  }

  .color-picker-dropdown {
    @apply flex flex-col w-48 border border-gray-800 absolute bg-gray-700 rounded-b-sm mt-1 shadow-xl;
  }

  .color-picker-selector {
    @apply flex flex-col w-full rounded-sm space-y-1;
  }

  .color-block-background {
    @apply h-48 w-full relative;
    background-color: var(--current-background);
  }

  .color-block-white-gradient {
    @apply h-full w-full bg-gradient-to-r from-white to-transparent;
  }

  .color-block-black-gradient {
    @apply h-full w-full bg-gradient-to-b from-transparent to-black;
  }

  .color-block-cursor {
    @apply h-4 w-4 rounded-full border-2 border-gray-700 absolute pointer-events-none;
    top: var(--color-block-top);
    left: var(--color-block-left);
    background: var(--current-rgba-color);
  }

  .color-picker-extra-selectors {
    @apply flex flex-col p-2 rounded-b-sm space-y-2;
  }

  .color-strip {
    @apply flex items-center h-4 w-full;
  }

  .color-strip-selector {
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

  .color-strip-selector::-moz-range-thumb {
    @apply w-3 h-3 cursor-pointer border-2 border-gray-700 rounded-full;
    background-color: var(--current-background);
  }

  .color-strip-selector::-webkit-slider-thumb {
    @apply bg-gray-800 w-4 h-4 cursor-pointer border-2 border-gray-700 rounded-full appearance-none;
  }

  .alpha-strip {
    @apply flex items-center h-4 w-full;
  }

  .alpha-strip-checkered-mask {
    @apply w-full h-2 rounded-full;
    background: repeating-conic-gradient(
        theme("colors.gray.400") 0deg 90deg,
        theme("colors.gray.700") 0deg 180deg
      )
      0 0 / theme("spacing.2");
  }

  .alpha-strip-opacity-mask {
    @apply w-full h-full relative rounded-full bg-gradient-to-r from-transparent;
  }

  .alpha-strip-selector {
    @apply bg-transparent h-2 absolute border-none rounded-full w-full appearance-none;
  }

  .alpha-strip-selector::-moz-range-thumb {
    @apply w-3 h-3 cursor-pointer border-2 border-gray-700 rounded-full;
    background-color: var(--current-background);
  }

  .alpha-strip-selector::-webkit-slider-thumb {
    @apply bg-gray-800 w-4 h-4 cursor-pointer border-2 border-gray-700 rounded-full appearance-none;
  }

  .inputs {
    @apply flex flex-row bg-gray-700 space-x-0.5;
  }
</style>
