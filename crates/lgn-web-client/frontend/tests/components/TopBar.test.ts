import TopBar from "../../src/components/TopBar.svelte";
import { render, fireEvent, cleanup } from "@testing-library/svelte";

describe("TopBar", () => {
  afterEach(() => cleanup());

  test("works", async () => {
    const { getByTestId } = render(TopBar);

    const file = getByTestId("menu-1");

    const dropDown = getByTestId("dropdown-1");

    expect(dropDown).toHaveClass("hidden");

    await fireEvent.click(file);

    expect(dropDown).not.toHaveClass("hidden");
  });
});
