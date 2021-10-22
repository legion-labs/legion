export function u32_to_hexcolor(v) {
    return "#" + v.toString(16);
}

export function hexcolor_to_u32(v) {
    return parseInt(v.substring(1), 16);
}

export default function () { };