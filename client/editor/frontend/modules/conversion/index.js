export function u32ToHexcolor(v) {
  return "#" + v.toString(16).padStart(8, "0");
}

export function hexcolorToU32(v) {
  return parseInt(v.substring(1), 16);
}
