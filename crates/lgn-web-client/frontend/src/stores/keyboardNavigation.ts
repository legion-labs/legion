import { Writable } from "../lib/store";

export type Value = {
  currentIndex: number | null;
};

export default class extends Writable<Value> {
  constructor() {
    super({ currentIndex: null });
  }
}
