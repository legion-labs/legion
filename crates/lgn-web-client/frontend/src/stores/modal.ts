import { SvelteComponentTyped } from "svelte";
import { Writable } from "../lib/store";

export class Content extends SvelteComponentTyped<{
  close(): void;
  payload: unknown;
}> {}

export type OpenModalEvent<Payload> = CustomEvent<{
  id: symbol;
  content: Content;
  payload?: Payload;
}>;

export type CloseModalEvent = CustomEvent<{
  id: symbol;
}>;

export type Value = Record<symbol, { content: Content; payload?: unknown }>;

export default class extends Writable<Value> {
  constructor() {
    super({});
  }

  /** Opens a modal with the provided content and payload */
  open(id: symbol, content: Content, payload?: unknown) {
    if (id in this.value) {
      return;
    }

    this.update((modals) => ({ ...modals, [id]: { content, payload } }));
  }

  /** Closes a modal */
  close(id: symbol) {
    if (!(id in this.value)) {
      return;
    }

    this.update((modals) => {
      const { [id]: _, ...restModals } = modals;

      return restModals;
    });
  }
}
