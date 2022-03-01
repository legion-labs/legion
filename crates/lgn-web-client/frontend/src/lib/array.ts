/** Adds the missing `filterMap` function that both `map` and `filter` an array in one iteration */
export function filterMap<T, U>(
  array: T[],
  f: (item: T, index: number, array: T[]) => U | null
): U[] {
  return array.reduce<U[]>((acc, value, index, array) => {
    const newValue = f(value, index, array);

    return newValue ? [...acc, newValue] : acc;
  }, []);
}

/** Take and accumulate an array elements into a new array as long as the provided predicate returns `true` */
export function takeWhile<T>(
  array: T[],
  pred: (item: T, index: number, array: T[]) => boolean
): T[] {
  if (!array.length) {
    return [];
  }

  function takeWhileIndex(slice: T[], index: number): T[] {
    if (!slice.length) {
      return [];
    }

    const [head, ...tail] = slice;

    return pred(head, index, array)
      ? [head, ...takeWhileIndex(tail, index + 1)]
      : [];
  }

  return takeWhileIndex(array, 0);
}

/** Removes an element from an array. Uses the `filter` method under the hood. */
export function remove<T>(array: T[], removedEntry: T) {
  return array.filter((entry) => entry !== removedEntry);
}

/**
 * Prepend an item to an array if the provided predicate function returns false.
 * Typically used to add an item to an array only if it's not already present.
 */
export function prependIfNonPresent<T>(
  array: T[],
  pred: (item: T, index: number, array: T[]) => boolean,
  item: () => T
): T[] {
  return array.some(pred) ? array : [item(), ...array];
}

export type NonEmptyArray<T> = [T, ...T[]];

export function isNonEmpty<T>(xs: T[]): xs is NonEmptyArray<T> {
  return xs.length ? true : false;
}
