import type { JsonValue } from "type-fest";

import { getCookie, removeCookie, setCookie } from "./cookie";

export interface Storage<Key, Value extends JsonValue> {
  get(key: Key): Value | null;
  has(key: Key): boolean;
  remove(key: Key): boolean;
  clear(): boolean;
  set(key: Key, value: Value): boolean;
  entries(): [Key, Value][];
  canStore(value: Value): boolean;
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

  canStore(_value: Value): boolean {
    // TODO: Check the size of the incoming value and return false if it's too large

    return true;
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

  canStore(_value: Value): boolean {
    // TODO: Check the size of the incoming value and return false if it's too large

    return true;
  }
}

export class DefaultCookie<Key extends string>
  implements Storage<Key, { value: string; expiresIn?: number }>
{
  get(key: Key): { value: string; expiresIn?: number | undefined } | null {
    const value = getCookie(key);

    if (value === null) {
      return null;
    }

    return { value };
  }

  has(key: Key): boolean {
    return this.get(key) !== null;
  }

  remove(key: Key): boolean {
    removeCookie(key);

    return true;
  }

  clear(): boolean {
    // We probably don't want to clear all the cookies at once programmatically, does nothing and returns false for now

    return false;
  }

  set(
    key: Key,
    { value, expiresIn }: { value: string; expiresIn?: number | undefined }
  ): boolean {
    setCookie(key, value, expiresIn);

    return true;
  }

  entries(): [Key, { value: string; expiresIn?: number | undefined }][] {
    const entries: [Key, { value: string; expiresIn?: number | undefined }][] =
      [];

    const parts = document.cookie.split(/[;=]/);

    for (let i = 0; i < parts.length - 1; i += 2) {
      entries.push([parts[i] as Key, { value: parts[i + 1] }]);
    }

    return entries;
  }

  canStore(_value: { value: string; expiresIn?: number | undefined }): boolean {
    // TODO: Check the size of the incoming value and return false if it's too large

    return true;
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

  canStore(_value: Value): boolean {
    return true;
  }
}
