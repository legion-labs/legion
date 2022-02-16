import { Writable } from "../lib/store";
import { Entry } from "../types/contextMenu";

export default class<Names extends string> extends Writable<
  Partial<Record<Names, Entry[]>>
> {
  constructor() {
    super({});
  }

  register(name: Names, entries: Entry[]): void {
    this.update((entrySets) => ({
      ...entrySets,
      [name]: entries,
    }));
  }
}
