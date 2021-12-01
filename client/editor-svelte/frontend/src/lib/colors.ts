// TODO: We could add value validations even though
// it's worth noticing that `color-convert` seems to handle well invalid values

import colorConvert from "color-convert";

// r: 0 ~ 255, g: 0 ~ 255, b: 0 ~ 255, a: 0 ~ 1
export type Rgba = { r: number; g: number; b: number; a: number };

// // r: 0 ~ 255, g: 0 ~ 255, b: 0 ~ 255
// export type Rgb = { r: number; g: number; b: number };

// h: 0 ~ 360, s: 0 ~ 100, v: 0 ~ 100, alpha: 0 ~ 100
export type Hsv = { h: number; s: number; v: number; alpha: number };

// Simple type alias for Hex colors (includes the #)
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

// export function parseHex(hex: string): Rgba | null {
//   if (hex.length === 4) {
//     return {
//       r: parseInt(hex.charAt(1), 16) * 0x11,
//       g: parseInt(hex.charAt(2), 16) * 0x11,
//       b: parseInt(hex.charAt(3), 16) * 0x11,
//       a: 1,
//     };
//   }

//   if (hex.length === 7) {
//     return {
//       r: parseInt(hex.substring(1, 3), 16),
//       g: parseInt(hex.substring(3, 5), 16),
//       b: parseInt(hex.substring(5, 7), 16),
//       a: 1,
//     };
//   }

//   return null;
// }

export function hsvToRgba({ h, s, v, alpha }: Hsv): Rgba {
  const [r, g, b] = colorConvert.hsv.rgb([h, s, v]);

  return { r, g, b, a: alpha / 100 };
}

export function rgbaToHsv({ r, g, b, a }: Rgba): Hsv {
  const [h, s, v] = colorConvert.rgb.hsv([r, g, b]);

  return { h, s, v, alpha: a * 100 };
}

export function rgbaToHex({ r, g, b }: Rgba) {
  return `#${colorConvert.rgb.hex([r, g, b]).toLowerCase()}`;
}

export function hsvToHex({ h, s, v }: Hsv) {
  return `#${colorConvert.hsv.hex([h, s, v]).toLowerCase()}`;
}

export function hexToHsv(hex: Hex): Hsv {
  const [h, s, v] = colorConvert.hex.hsv(hex.slice(1));

  return { h, s, v, alpha: 100 };
}

export function hexToRgba(hex: Hex): Rgba {
  const [r, g, b] = colorConvert.hex.rgb(hex.slice(1));

  return { r, g, b, a: 1 };
}

export function rgbaToColorString({ r, g, b, a }: Rgba, ignoreAlpha = false) {
  return `rgba(${r},${g},${b},${ignoreAlpha ? 1 : a.toPrecision(2)})`;
}

export function hsvToColorString(hsv: Hsv, ignoreAlpha = false): string {
  const rgba = hsvToRgba(hsv);

  return rgbaToColorString(rgba, ignoreAlpha);
}
