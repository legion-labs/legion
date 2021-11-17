export function u32ToHexcolor(v: number) {
  return "#" + v.toString(16).padStart(8, "0");
}

export function hexcolorToU32(v: string) {
  const colorValue = parseInt(v.substring(1), 16);

  return isNaN(colorValue) ? null : colorValue;
}
