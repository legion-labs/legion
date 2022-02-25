import Sortable from "sortablejs";

export default function sortable(
  element: HTMLElement,
  options?: Sortable.Options
) {
  Sortable.create(element, options);
}
