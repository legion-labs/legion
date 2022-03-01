import { Entries } from "@/lib/hierarchyTree";
import resources from "@/resources/resourcesResponse.json";

describe("updateEntry", () => {
  test("updates no entry names when the provided function always returns null in the Entries", () => {
    const entries = Entries.fromArray(resources);

    expect(entries.update(() => null)).toEqual(entries);
  });

  test("updates no entry names when the provided function always return an empty string in the Entries", () => {
    const entries = Entries.fromArray(resources);

    expect(
      entries.update((entry) => ({
        ...entry,
        name: "",
      }))
    ).toEqual(entries);
  });

  test('updates 1 entry name "leaf" when the provided function returns a non empty string for a "leaf" entry', () => {
    const entries = Entries.fromArray(resources);

    expect(
      entries.update((entry) =>
        entry.name === "cube_group.ent"
          ? { ...entry, name: "new_cube_group.ent" }
          : null
      )
    ).toMatchSnapshot();
  });

  test('updates 1 entry name "node" when the provided function returns a non empty string for a "node" entry', () => {
    const entries = Entries.fromArray(resources);

    expect(
      entries.update((entry) =>
        entry.name === "world" ? { ...entry, name: "monde" } : null
      )
    ).toMatchSnapshot();
  });
});

describe("fromArray", () => {
  test("transforms a `ResourceDescription` array into a hierarchical tree", () => {
    expect(Entries.fromArray(resources)).toMatchSnapshot();
  });
});
