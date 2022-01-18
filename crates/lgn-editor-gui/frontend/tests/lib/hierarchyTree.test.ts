import { unflatten, updateEntry } from "@/lib/hierarchyTree";
import resources from "@/resources/resourcesResponse.json";

describe("updateEntry", () => {
  it("Updates no entry names when the provided function always returns null in the Entries", () => {
    const entries = unflatten(resources);

    expect(updateEntry(entries, () => null)).toEqual(entries);
  });

  it("Updates no entry names when the provided function always return an empty string in the Entries", () => {
    const entries = unflatten(resources);

    expect(
      updateEntry(entries, () => ({
        name: "",
      }))
    ).toEqual(entries);
  });

  it('Updates 1 entry name "leaf" when the provided function returns a non empty string for a "leaf" entry', () => {
    const entries = unflatten(resources);

    expect(
      updateEntry(entries, (name) =>
        name === "cube_group.ent" ? { name: "new_cube_group.ent" } : null
      )
    ).toMatchSnapshot();
  });

  it('Updates 1 entry name "node" when the provided function returns a non empty string for a "node" entry', () => {
    const entries = unflatten(resources);

    expect(
      updateEntry(entries, (name) =>
        name === "world" ? { name: "monde" } : null
      )
    ).toMatchSnapshot();
  });
});

describe("unflatten", () => {
  it("Unflattens a `ResourceDescription` array into a hierarchical tree", () => {
    expect(unflatten(resources)).toMatchSnapshot();
  });
});
