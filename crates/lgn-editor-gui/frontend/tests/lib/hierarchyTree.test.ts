import { Entries } from "@/lib/hierarchyTree";
import resources from "@/resources/resourcesResponse.json";

describe("updateEntry", () => {
  it("updates no entry names when the provided function always returns null in the Entries", () => {
    const entries = Entries.fromArray(resources, Symbol);

    expect(entries.update(() => null)).toEqual(entries);
  });

  it("updates no entry names when the provided function always return an empty string in the Entries", () => {
    const entries = Entries.fromArray(resources, Symbol);

    expect(
      entries.update((entry) => ({
        ...entry,
        name: "",
      }))
    ).toEqual(entries);
  });

  it('updates 1 entry name "leaf" when the provided function returns a non empty string for a "leaf" entry', () => {
    const entries = Entries.fromArray(resources, Symbol);

    expect(
      entries.update((entry) =>
        entry.name === "cube_group.ent"
          ? { ...entry, name: "new_cube_group.ent" }
          : null
      )
    ).toMatchSnapshot();
  });

  it('updates 1 entry name "node" when the provided function returns a non empty string for a "node" entry', () => {
    const entries = Entries.fromArray(resources, Symbol);

    expect(
      entries.update((entry) =>
        entry.name === "world" ? { ...entry, name: "monde" } : null
      )
    ).toMatchSnapshot();
  });
});

describe("fromArray", () => {
  it("transforms a `ResourceDescription` array into a hierarchical tree", () => {
    expect(Entries.fromArray(resources, Symbol)).toMatchSnapshot();
  });
});
