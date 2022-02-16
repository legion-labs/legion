import { SvelteComponentTyped } from "svelte";
import { Writable } from "../lib/store";

export type Config<Payload = unknown> = {
  payload?: Payload;
  noTransition?: boolean;
};

export class Content extends SvelteComponentTyped<{
  close?(): void;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  config?: Config<any>;
}> {}

export type Value = Record<
  symbol,
  {
    content: typeof Content;
    config?: Config;
  }
>;

export default class extends Writable<Value> {
  constructor() {
    super({});
  }

  /** Opens a modal with the provided content and payload */
  open(id: symbol, content: typeof Content, config?: Config) {
    if (id in this.value) {
      return;
    }

    this.update((modals) => ({
      ...modals,
      [id]: { content, config },
    }));
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
