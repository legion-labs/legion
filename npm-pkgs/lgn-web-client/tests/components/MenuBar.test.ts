import { cleanup, fireEvent, render } from "@testing-library/svelte";

import MenuBar from "../../src/components/menu/MenuBar.svelte";
import type { MenuItemDescription } from "../../src/components/menu/lib/MenuItemDescription";

describe("MenuBar", () => {
  afterEach(() => cleanup());

  test("works", async () => {
    const { container } = render(MenuBar, {
      items: [
        {
          title: "Root",
          children: [
            {
              title: "Child",
            },
          ],
        } as MenuItemDescription,
      ],
    });

    const menuRoot = container.getElementsByClassName("menu-root")[0];

    await fireEvent.click(menuRoot);

    const dropDown = menuRoot.getElementsByClassName("menu-dropdown")[0];

    expect(dropDown).not.toHaveClass("hidden");
  });
});
