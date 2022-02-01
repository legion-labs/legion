import { SvelteComponent } from "svelte";

export type OpenModalEvent = CustomEvent<{ content: SvelteComponent }>;

/** Opens a modal with the provided content */
export function openModal(content: SvelteComponent) {
  window.dispatchEvent(new CustomEvent("open-modal", { detail: { content } }));
}

/** Closes the currently opened modal */
export function closeModal() {
  window.dispatchEvent(new CustomEvent("close-modal"));
}
