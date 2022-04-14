export const spanPixelHeight = 20;

export const threadItemLength = getThreadItemLength();

export const pixelMargin = 4;

export const asyncTaskName = "async tasks";

function getThreadItemLength() {
  return 170;
  const style = getComputedStyle(document.body);
  const item = style.getPropertyValue("--thread-item-length");
  return Number.parseInt(item.replace("px", ""));
}
