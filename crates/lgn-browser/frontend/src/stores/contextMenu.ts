import { Writable } from "../lib/store";
import { Entry } from "../types/contextMenu";

export default class<
  EntryRecord extends Record<string, unknown>
> extends Writable<EntryRecord> {
  constructor() {
    super({} as EntryRecord);
  }

  register<Name extends keyof EntryRecord>(name: Name, entries: Entry[]): void {
    this.update((entrySets) => ({
      ...entrySets,
      [name]: entries,
    }));
  }
}
