import "@testing-library/jest-dom";
import fetch from "node-fetch";
import { vi } from "vitest";

globalThis.fetch = fetch as typeof globalThis.fetch;

vi.mock("uuid", () => ({
  v4: () => "same-old-id",
}));
