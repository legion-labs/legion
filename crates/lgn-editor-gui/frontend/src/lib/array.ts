/** Adds the missing `filterMap` function that both `map` and `filter` an array in one iteration */
export function filterMap<T, U>(
  array: T[],
  f: (arg: T, index: number, array: T[]) => U | null
): U[] {
  return array.reduce<U[]>((acc, value, index, array) => {
    const newValue = f(value, index, array);

    return newValue ? [...acc, newValue] : acc;
  }, []);
}
