import { SvelteComponentTyped } from "svelte";
import { get, writable } from "svelte/store";
import Prompt from "../components/modal/Prompt.svelte";

export type Payload = Record<string, unknown>;

export type Config<P = Payload> = {
  payload?: P;
  noTransition?: boolean;
};

// TODO: Improve typings
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export class Content extends SvelteComponentTyped<any> {}

export type Value = Record<
  symbol,
  {
    id: symbol;
    content: typeof Content;
    config?: Config;
  }
>;

export function createModalStore() {
  return {
    ...writable<Value>({}),

    /** Opens a modal with the provided content and payload */
    open(id: symbol, content: typeof Content, config?: Config) {
      if (id in get(this)) {
        return;
      }

      this.update((modals) => ({
        ...modals,
        [id]: { content, config, id },
      }));
    },

    /** Opens a prompt modal */
    prompt(
      id: symbol,
      config?: Config<{
        title?: string;
        message?: string;
        cancelLabel?: string;
        confirmLabel?: string;
      }>
    ) {
      this.open(id, Prompt, config);
    },

    /** Closes a modal */
    close(id: symbol) {
      if (!(id in get(this))) {
        return;
      }

      this.update((modals) => {
        const { [id]: _, ...restModals } = modals;

        return restModals;
      });
    },
  };
}
