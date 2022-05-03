import { remToPx } from "@lgn/web-client/src/lib/html";

export const spanPixelHeight = 20;

export const pixelMargin = 4;

export const asyncTaskName = "async tasks";

export function getThreadItemLength(): number {
  const value = getComputedStyle(document.body)
    .getPropertyValue("--thread-item-length")
    .trim();

  const rem = value.match(/^(\d+)rem$/)?.[1];

  if (!rem) {
    throw new Error(
      `The --thread-item-length CSS variable is not set or not set properly, it should be provided using the rem unit, was "${value}"`
    );
  }

  const px = remToPx(+rem);

  if (px === null) {
    throw new Error(
      `Unable to convert from rem to px the "--thread-item-length" CSS veriable with value "${value}"`
    );
  }

  return px;
}
