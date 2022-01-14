import currentResource from "@/stores/currentResource";
import PropertyGrid from "@/components/propertyGrid/PropertyGrid.svelte";
import { render } from "@testing-library/svelte";
import properties from "@/resources/propertiesResponse.json";
import { formatProperties, ResourceProperty } from "@/lib/propertyGrid";

describe("PropertyGrid", () => {
  it("renders properly without resources", () => {
    const { container } = render(PropertyGrid);

    expect(container).toMatchSnapshot();
  });

  it("renders properly with an error", () => {
    currentResource.error.set("Ooops, an error occured");

    const { container } = render(PropertyGrid);

    expect(container).toMatchSnapshot();

    currentResource.error.set(null);
  });

  // it("renders properly with the current resource set", () => {
  //   currentResource.data.set({
  //     id: "id",
  //     description: {
  //       id: "id",
  //       path: "",
  //       version: 1,
  //     },
  //     version: 1,
  //     properties: formatProperties(properties as unknown as ResourceProperty[]),
  //   });

  //   const { container } = render(PropertyGrid);

  //   expect(container).toMatchSnapshot();
  // });
});
