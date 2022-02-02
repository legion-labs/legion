import { Writable } from "../lib/store";

export const menus = [
  { id: 1, title: "File" },
  { id: 2, title: "Edit" },
  { id: 3, title: "Layer" },
  { id: 4, title: "Document" },
  { id: 5, title: "View" },
  { id: 6, title: "Help" },
] as const;

// Not meant to be used as is in production
// as the menu might become dynamic at one point
export type Id = typeof menus[number]["id"];

class Store extends Writable<Id | null> {
  constructor() {
    super(null);
  }

  close() {
    super.set(null);
  }

  override set(id: Id) {
    super.set(id);
  }
}

export default new Store();
