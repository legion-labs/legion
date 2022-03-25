export const spanPixelHeight = 20;

export const threadItemLength = getThreadItemLength();

function getThreadItemLength() {
  const style = getComputedStyle(document.body);
  const item = style.getPropertyValue("--thread-item-length");
  return Number.parseInt(item.replace("px", ""));
}
