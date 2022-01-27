import { filterMap, takeWhile } from "@/lib/array";

describe("filterMap", () => {
  it("returns an empty array when the provided array is empty", () => {
    expect(filterMap([], () => null)).toEqual([]);
  });

  it("returns an empty array when the provided array is not empty but the provided function returns always `null`", () => {
    expect(filterMap([1, 2, 3], () => null)).toEqual([]);
  });

  it("returns an array of even number multiplied by 2 when the provided function returns `null` if a number is odd and the number multiplied by 2 if the number is even", () => {
    expect(
      filterMap([1, 2, 3, 4, 5, 6], (x) => (x % 2 === 0 ? x * 2 : null))
    ).toEqual([4, 6, 8, 12]);
  });
});

describe("takeWhile", () => {
  it("returns an empty array when the provided array is empty", () => {
    expect(takeWhile([], () => false)).toEqual([]);
  });

  it("returns an empty array when the provided array is not empty but the provided function returns always `false`", () => {
    expect(takeWhile([1, 2, 3], () => false)).toEqual([]);
  });

  it("returns an array containing only the first even numbers when the provided function returns always `true` when the number is even and `false` when it's odd", () => {
    expect(takeWhile([2, 4, 6, 7, 8, 10], (x) => x % 2 === 0)).toEqual([
      2, 4, 6,
    ]);
  });
});
