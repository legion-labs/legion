import { SvelteComponentTyped } from "svelte";
import { get, Writable, writable } from "svelte/store";
import Prompt from "../components/modal/Prompt.svelte";

export type Payload = Record<string, unknown>;

export type Config<P = Payload> = {
  payload?: P;
  noTransition?: boolean;
};

// TODO: Improve typings
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export class Content extends SvelteComponentTyped<any> {}

export type ModalValue = Record<
  symbol,
  {
    id: symbol;
    content: typeof Content;
    config?: Config;
  }
>;

export type ModalStore = Writable<ModalValue> & {
  open(id: symbol, content: typeof Content, config?: Config): void;
  prompt(
    id: symbol,
    config?: Config<{
      title?: string;
      message?: string;
      cancelLabel?: string;
      confirmLabel?: string;
    }>
  ): void;
  close(id: symbol): void;
};

export function createModalStore(): ModalStore {
  return {
    ...writable<ModalValue>({}),

    /** Opens a modal with the provided content and payload */
    open(id, content, config?) {
      if (id in get(this)) {
        return;
      }

      this.update((modals) => ({
        ...modals,
        [id]: { content, config, id },
      }));
    },

    /** Opens a prompt modal */
    prompt(id, config?) {
      this.open(id, Prompt, config);
    },

    /** Closes a modal */
    close(id) {
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
