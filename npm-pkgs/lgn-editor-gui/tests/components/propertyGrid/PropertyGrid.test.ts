import { cleanup } from "@testing-library/svelte";
import { render } from "@testing-library/svelte";

import type { PropertyInspector } from "@lgn/apis/editor";

import PropertyGrid from "@/components/propertyGrid/PropertyGrid.svelte";
import { formatProperties } from "@/components/propertyGrid/lib/propertyGrid";
import currentResource from "@/orchestrators/currentResource";
import properties from "@/resources/propertiesResponse.json";

describe("PropertyGrid", () => {
  afterEach(() => cleanup());

  test("renders properly without resources", () => {
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
        type: "",
      },
      version: 1,
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      properties: formatProperties(
        properties as unknown as PropertyInspector.ResourceProperty[]
      ),
    });

    const { container } = render(PropertyGrid);

    expect(container).toMatchSnapshot();
  });
});
