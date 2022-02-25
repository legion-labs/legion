import currentResource from "@/stores/currentResource";
import PropertyGrid from "@/components/propertyGrid/PropertyGrid.svelte";
import { cleanup } from "@testing-library/svelte";
import { render } from "@testing-library/svelte";
import properties from "@/resources/propertiesResponse.json";
import { formatProperties, ResourceProperty } from "@/lib/propertyGrid";

describe("PropertyGrid", () => {
  afterEach(() => cleanup());

  test("renders properly without resources", async () => {
    const { container } = render(PropertyGrid);

    expect(container).toMatchSnapshot();
  });

  test("renders properly with an error", () => {
    currentResource.error.set("Ooops, an error occured");

    const { container } = render(PropertyGrid);

    expect(container).toMatchSnapshot();

    currentResource.error.set(null);
  });

  test("renders properly with the current resource set", () => {
    currentResource.data.set({
      id: "id",
      description: {
        id: "id",
        path: "",
        version: 1,
      },
      version: 1,
      properties: formatProperties(properties as unknown as ResourceProperty[]),
    });

    const { container } = render(PropertyGrid);

    expect(container).toMatchSnapshot();
  });
});
