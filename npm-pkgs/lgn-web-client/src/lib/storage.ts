import type { JsonValue } from "type-fest";

export interface Storage<Key, Value extends JsonValue> {
  get(key: Key): Value | null;
  has(key: Key): boolean;
  remove(key: Key): boolean;
  clear(): boolean;
  set(key: Key, value: Value): boolean;
  entries(): [Key, Value][];
}

export class DefaultLocalStorage<Key extends string, Value extends JsonValue>
  implements Storage<Key, Value>
{
  get(key: Key): Value | null {
    const value = globalThis.localStorage.getItem(key);

    if (value === null) {
      return null;
    }

    try {
      return JSON.parse(value) as Value;
    } catch {
      return null;
    }
  }

  has(key: Key): boolean {
    return globalThis.localStorage.getItem(key) !== null;
  }

  remove(key: Key): boolean {
    globalThis.localStorage.removeItem(key);

    return !this.has(key);
  }

  clear(): boolean {
    globalThis.localStorage.clear();

    return true;
  }

  set(key: Key, value: Value): boolean {
    try {
      globalThis.localStorage.setItem(key, JSON.stringify(value));

      return true;
    } catch {
      return false;
    }
  }

  entries(): [Key, Value][] {
    const entries: [Key, Value][] = [];

    for (let i = 0; i <= globalThis.localStorage.length; i++) {
      const key = globalThis.localStorage.key(i);

      if (key !== null) {
        const value = this.get(key as Key);

        if (value !== null) {
          entries.push([key as Key, value]);
        }
      }
    }

    return entries;
  }
}

export class DefaultSessionStorage<Key extends string, Value extends JsonValue>
  implements Storage<Key, Value>
{
  get(key: Key): Value | null {
    const value = globalThis.sessionStorage.getItem(key);

    if (value === null) {
      return null;
    }

    try {
      return JSON.parse(value) as Value;
    } catch {
      return null;
    }
  }

  has(key: Key): boolean {
    return globalThis.sessionStorage.getItem(key) !== null;
  }

  remove(key: Key): boolean {
    globalThis.sessionStorage.removeItem(key);

    return !this.has(key);
  }

  clear(): boolean {
    globalThis.sessionStorage.clear();

    return true;
  }

  set(key: Key, value: Value): boolean {
    try {
      globalThis.sessionStorage.setItem(key, JSON.stringify(value));

      return true;
    } catch {
      return false;
    }
  }

  entries(): [Key, Value][] {
    const entries: [Key, Value][] = [];

    for (let i = 0; i <= globalThis.sessionStorage.length; i++) {
      const key = globalThis.sessionStorage.key(i);

      if (key !== null) {
        const value = this.get(key as Key);

        if (value !== null) {
          entries.push([key as Key, value]);
        }
      }
    }

    return entries;
  }
}

export class InMemory<Key extends string, Value extends JsonValue>
  implements Storage<Key, Value>
{
  #record: Record<Key, string> = {} as Record<Key, string>;

  get(key: Key): Value | null {
    const value = this.#record[key];

    if (value === undefined) {
      return null;
    }

    try {
      return JSON.parse(value) as Value;
    } catch {
      return null;
    }
  }

  has(key: Key): boolean {
    return key in this.#record;
  }

  remove(key: Key): boolean {
    delete this.#record[key];

    return !this.has(key);
  }

  clear(): boolean {
    this.#record = {} as Record<Key, string>;

    return true;
  }

  set(key: Key, value: Value): boolean {
    try {
      this.#record[key] = JSON.stringify(value);

      return true;
    } catch {
      return false;
    }
  }

  entries(): [Key, Value][] {
    return Object.entries(this.#record) as [Key, Value][];
  }
}
