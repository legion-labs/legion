export type Phantom<T, U> = T & {
  /**
   * @deprecated @inline Don't use this attribute, it's value is always `null`!
   *
   * Its presence is due to the fact TypeScript doesn't support
   * [nominal typing](https://github.com/Microsoft/TypeScript/issues/202),
   * so we need to _structurally_ change the type and the value.
   *
   * You can see an example of why it matters here: https://tsplay.dev/N5460w
   */
  readonly "#phantom": U;
};

/**
 * Creates a pseudo ["phantom type"](https://wiki.haskell.org/Phantom_type) from an object value.
 *
 * _The `#phantom` attribute is always `null` and not meant to be used as is._
 */
export default function phantom<T, U>(data: T): Phantom<T, U> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return { ...data, "#phantom": null as any as U };
}

/**
 * Convenient function that maps an array to create an array of phantom values
 */
export function phantoms<T, U>(data: T[]): Phantom<T, U>[] {
  return data.map<Phantom<T, U>>(phantom);
}
