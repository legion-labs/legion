// TODO: We could add value validations even though
// `color-convert` seems to handle invalid values very well

import colorConvert from "color-convert";

// r: 0 ~ 255, g: 0 ~ 255, b: 0 ~ 255, a: 0 ~ 255
export type Rgba = { r: number; g: number; b: number; a: number };

// h: 0 ~ 360, s: 0 ~ 100, v: 0 ~ 100, alpha: 0 ~ 255
export type Hsv = { h: number; s: number; v: number; a: number };

// Simple type alias for Hex colors
export type Hex = string;

export type ColorSet = { hsv: Hsv; rgba: Rgba; hex: Hex };

// Hue goes from 0° to 360°
export const maxHueValue = 360;

export function colorSetFromRgba(rgba: Rgba): ColorSet {
  return { hsv: rgbaToHsv(rgba), rgba, hex: rgbaToHex(rgba) };
}

export function colorSetFromHsv(hsv: Hsv): ColorSet {
  return { hsv, rgba: hsvToRgba(hsv), hex: hsvToHex(hsv) };
}

export function colorSetFromHex(hex: Hex): ColorSet {
  return { hsv: hexToHsv(hex), rgba: hexToRgba(hex), hex };
}

/**
 * Parses an rgba string of format `rgba(100, 0, 200, 0.5)`
 * @param rgba An rgba string
 * @returns An Rgba object
 */
export function parseRgba(rgba: string): Rgba {
  const [r, g, b, a] = rgba
    .slice(5)
    .slice(0, -1)
    .split(",")
    .map((colorPart) => +colorPart.trim());

  return { r, g, b, a };
}

export function hsvToRgba({ h, s, v, a }: Hsv): Rgba {
  const [r, g, b] = colorConvert.hsv.rgb([h, s, v]);

  return { r, g, b, a };
}

export function rgbaToHsv({ r, g, b, a }: Rgba): Hsv {
  const [h, s, v] = colorConvert.rgb.hsv([r, g, b]);

  return { h, s, v, a };
}

export function rgbaToHex({ r, g, b, a }: Rgba) {
  return `${colorConvert.rgb.hex([r, g, b]).toLowerCase()}${a
    .toString(16)
    .padStart(2, "0")}`;
}

export function hsvToHex({ h, s, v, a }: Hsv) {
  return `${colorConvert.hsv.hex([h, s, v]).toLowerCase()}${a
    .toString(16)
    .padStart(2, "0")}`;
}

export function hexToHsv(hex: Hex): Hsv {
  const [h, s, v] = colorConvert.hex.hsv(hex.slice(0, 6));

  return { h, s, v, a: parseInt(hex.slice(6, 8), 16) };
}

export function hexToRgba(hex: Hex): Rgba {
  const [r, g, b] = colorConvert.hex.rgb(hex);

  return { r, g, b, a: parseInt(hex.slice(6, 8), 16) };
}

export function rgbaToColorString({ r, g, b, a }: Rgba, ignoreAlpha = false) {
  return `rgba(${r},${g},${b},${ignoreAlpha ? 1 : (a / 255).toPrecision(2)})`;
}

export function hsvToColorString(hsv: Hsv, ignoreAlpha = false): string {
  const rgba = hsvToRgba(hsv);

  return rgbaToColorString(rgba, ignoreAlpha);
}
