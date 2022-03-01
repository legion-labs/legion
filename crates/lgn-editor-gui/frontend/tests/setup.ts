import { vi } from "vitest";
import "@testing-library/jest-dom";

vi.mock("uuid", () => ({
  v4: () => "same-old-id",
}));
