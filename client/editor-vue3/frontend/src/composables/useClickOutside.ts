import { Ref, ref } from "@vue/reactivity";
import { onMounted, onUnmounted } from "@vue/runtime-core";

/**
 * Basic click outside composable, takes a function that will be called
 * when the user clicks outside the targeted html element.
 * The callbacks receives the `MouseEvent` as a parameter if
 * extra checks are required.
 * Returns the ref to use to target an html element.
 */
export default function useClickOutside(f: (event: MouseEvent) => void) {
  const elementRef: Ref<HTMLElement | null> = ref(null);

  const listener = (event: MouseEvent) => {
    if (
      elementRef.value &&
      event.target instanceof Node &&
      !elementRef.value.contains(event.target)
    ) {
      f(event);
    }
  };

  onMounted(() => {
    window.addEventListener("click", listener);
  });

  onUnmounted(() => {
    window.removeEventListener("click", listener);
  });

  return { ref: elementRef };
}
